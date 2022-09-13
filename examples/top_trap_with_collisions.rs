//! Time-Orbiting Potential (TOP) trap with collisions

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::collisions::CollisionPlugin;
use lib::collisions::{ApplyCollisionsOption, CollisionParameters, CollisionsTracker};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
// use lib::magnetic::force::{ApplyMagneticForceSystem, MagneticDipole};
use lib::magnetic::force::{MagneticDipole};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::magnetic::top::TimeOrbitingPotential;
use lib::magnetic::MagneticTrapPlugin;
use lib::output::file::FileOutputPlugin;
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
use specs::prelude::*;
use std::fs::File;
use std::io::{Error, Write};

fn main() {
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugin(FileOutputPlugin::<Position, Text, Atom>::new(
        "pos.txt".to_string(),
        100,
    ));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new(
        "vel.txt".to_string(),
        100,
    ));

    // Add magnetics systems (todo: as plugin)
    sim_builder.world.register::<NewlyCreated>();
    sim_builder.add_plugin(MagneticTrapPlugin);
    sim_builder.add_end_frame_systems();
    sim_builder.add_plugin(CollisionPlugin);

    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(80.0, Vector3::z()))
        .with(Position::new())
        .build();

    sim.world
        .create_entity()
        .with(TimeOrbitingPotential::gauss(20.0, 3000.0)) // Time averaged TOP theory assumes rotation frequency much greater than velocity of atoms
        .build();

    let p_dist = Normal::new(0.0, 50e-6).unwrap();
    let v_dist = Normal::new(0.0, 0.004).unwrap(); // ~100nK

    for _i in 0..25000 {
        sim.world
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

    sim.world.insert(ApplyCollisionsOption);
    sim.world.insert(CollisionParameters {
        macroparticle: 4e2,
        box_number: 200, //Any number large enough to cover entire cloud with collision boxes. Overestimating box number will not affect performance.
        box_width: 20e-6, //Too few particles per box will both underestimate collision rate and cause large statistical fluctuations.
        //Boxes must also be smaller than typical length scale of density variations within the cloud, since the collisions model treats gas within a box as homogeneous.
        sigma: 3.5e-16, //Approximate collisional cross section of Rb87
        collision_limit: 10_000_000.0, //Maximum number of collisions that can be calculated in one frame.
                                       //This avoids absurdly high collision numbers if many atoms are initialised with the same position, for example.
    });
    sim.world.insert(CollisionsTracker {
        num_collisions: Vec::new(),
        num_atoms: Vec::new(),
        num_particles: Vec::new(),
    });

    // Define timestep
    sim.world.insert(Timestep { delta: 5e-5 }); //Aliasing of TOP field or other strange effects can occur if timestep is not much smaller than TOP field period.
                                                //Timestep must also be much smaller than mean collision time.

    let mut filename = File::create("collisions.txt").expect("Cannot create file.");

    // Run the simulation for a number of steps.
    for _i in 0..10000 {
        sim.step();

        if (_i > 0) && (_i % 50_i32 == 0) {
            let tracker = sim.world.read_resource::<CollisionsTracker>();
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
