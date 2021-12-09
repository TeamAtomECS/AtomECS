//! Time-Orbiting Potential (TOP) trap with collisions

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::collisions::{
    ApplyCollisionsOption, ApplyCollisionsSystem, CollisionParameters, CollisionsTracker,
};
use lib::ecs;
use lib::ecs::AtomecsDispatcherBuilder;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::integrator::INTEGRATE_VELOCITY_SYSTEM_NAME;
use lib::magnetic::force::{ApplyMagneticForceSystem, MagneticDipole};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::magnetic::top::TimeOrbitingPotential;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
use specs::prelude::*;
use std::fs::File;
use std::io::{Error, Write};

fn main() {
    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    world.register::<NewlyCreated>();
    world.register::<MagneticDipole>();
    world.register::<TimeOrbitingPotential>();
    let mut atomecs_builder = AtomecsDispatcherBuilder::new();
    atomecs_builder.add_frame_initialisation_systems();
    atomecs_builder.add_systems();
    atomecs_builder.builder.add(
        ApplyMagneticForceSystem {},
        "magnetic_force",
        &["magnetics_gradient"],
    );
    atomecs_builder.builder.add(
        ApplyCollisionsSystem {},
        "collisions",
        &[INTEGRATE_VELOCITY_SYSTEM_NAME], // Collisions system must be applied after velocity integrator or it will violate conservation of energy and cause heating
    );

    atomecs_builder.add_frame_end_systems();
    let mut builder = atomecs_builder.builder;

    // Configure simulation output.
    builder = builder.with(
        file::new::<Position, Text>("pos.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(80.0, Vector3::z()))
        .with(Position::new())
        .build();

    world
        .create_entity()
        .with(TimeOrbitingPotential::gauss(20.0, 3000.0)) // Time averaged TOP theory assumes rotation frequency much greater than velocity of atoms
        .build();

    let p_dist = Normal::new(0.0, 50e-6).unwrap();
    let v_dist = Normal::new(0.0, 0.004).unwrap(); // ~100nK

    for _i in 0..25000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(
                    p_dist.sample(&mut rand::thread_rng()),
                    p_dist.sample(&mut rand::thread_rng()),
                    0.35 * p_dist.sample(&mut rand::thread_rng()), //TOP traps have tighter confinement along quadrupole axis
                ),
            })
            .with(Atom)
            .with(Force::new())
            .with(Velocity {
                vel: Vector3::new(
                    v_dist.sample(&mut rand::thread_rng()),
                    v_dist.sample(&mut rand::thread_rng()),
                    v_dist.sample(&mut rand::thread_rng()),
                ),
            })
            .with(NewlyCreated)
            .with(MagneticDipole { mFgF: 0.5 })
            .with(Mass { value: 87.0 })
            .build();
    }

    world.insert(ApplyCollisionsOption);
    world.insert(CollisionParameters {
        macroparticle: 4e2,
        box_number: 200, //Any number large enough to cover entire cloud with collision boxes. Overestimating box number will not affect performance.
        box_width: 20e-6, //Too few particles per box will both underestimate collision rate and cause large statistical fluctuations.
        //Boxes must also be smaller than typical length scale of density variations within the cloud, since the collisions model treats gas within a box as homogeneous.
        sigma: 3.5e-16, //Approximate collisional cross section of Rb87
        collision_limit: 10_000_000.0, //Maximum number of collisions that can be calculated in one frame.
                                       //This avoids absurdly high collision numbers if many atoms are initialised with the same position, for example.
    });
    world.insert(CollisionsTracker {
        num_collisions: Vec::new(),
        num_atoms: Vec::new(),
        num_particles: Vec::new(),
    });

    // Define timestep
    world.insert(Timestep { delta: 5e-5 }); //Aliasing of TOP field or other strange effects can occur if timestep is not much smaller than TOP field period.
                                            //Timestep must also be much smaller than mean collision time.

    let mut filename = File::create("collisions.txt").expect("Cannot create file.");

    // Run the simulation for a number of steps.
    for _i in 0..10000 {
        dispatcher.dispatch(&mut world);
        world.maintain();

        if (_i > 0) && (_i % 50_i32 == 0) {
            let tracker = world.read_resource::<CollisionsTracker>();
            let _result = write_collisions_tracker(
                &mut filename,
                &_i,
                &tracker.num_collisions,
                &tracker.num_atoms,
                &tracker.num_particles,
            )
            .expect("Could not write collision stats file.");
        }
    }
}

// // Write collision stats to file

fn write_collisions_tracker(
    filename: &mut File,
    step: &i32,
    num_collisions: &Vec<i32>,
    num_atoms: &Vec<f64>,
    num_particles: &Vec<i32>,
) -> Result<(), Error> {
    let str_collisions: Vec<String> = num_collisions.iter().map(|n| n.to_string()).collect();
    let str_atoms: Vec<String> = num_atoms.iter().map(|n| format!("{:.2}", n)).collect();
    let str_particles: Vec<String> = num_particles.iter().map(|n| n.to_string()).collect();
    write!(
        filename,
        "{:?}\r\n{:}\r\n{:}\r\n{:}\r\n",
        step,
        str_collisions.join(" "),
        str_atoms.join(" "),
        str_particles.join(" ")
    )?;
    Ok(())
}
