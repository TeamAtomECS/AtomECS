//! Loading a Rb pyramid MOT from vapor
//!
//! One of the beams has a circular mask, which allows atoms to escape on one side.

extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::force::RandomWalkMarker;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use specs::{Builder, World};
use std::time::Instant;

fn main() {
    let now = Instant::now();

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 10),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(18.2, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    // Create cooling lasers.
    let detuning = -3.0;
    let power = 0.1;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 0.0, 1.0),
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            -1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 0.0, -1.0),
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            -1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, 0.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 1.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            1.0,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, -1.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            1.0,
        ))
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 0.3e-6 });
    world.add_resource(RandomWalkMarker { value: true });

    // Add atoms
    for _ in 0..1000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0001),
            })
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.01),
            })
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(AtomInfo::rubidium())
            .with(Atom)
            .with(NewlyCreated)
            .build();
    }

    // Run the simulation for a number of steps.
    for _i in 0..6000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
