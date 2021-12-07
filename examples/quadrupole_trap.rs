//! An example of trapping atoms in a quadrupole trap.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::ecs::AtomecsDispatcherBuilder;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::magnetic::force::{ApplyMagneticForceSystem, MagneticDipole};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use rand_distr::{Distribution, Normal};

use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use specs::prelude::*;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    world.register::<NewlyCreated>();
    world.register::<MagneticDipole>();
    let mut atomecs_builder = AtomecsDispatcherBuilder::new();
    atomecs_builder.add_frame_initialisation_systems();
    atomecs_builder.add_systems();
    atomecs_builder.builder.add(
        ApplyMagneticForceSystem {},
        "magnetic_force",
        &["magnetics_gradient"],
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
        .with(QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()))
        .with(Position::new())
        .build();

    let p_dist = Normal::new(0.0, 0.5e-3).unwrap();
    let v_dist = Normal::new(0.0, 0.09).unwrap(); //80uK

    for _i in 0..1000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(
                    p_dist.sample(&mut rand::thread_rng()),
                    p_dist.sample(&mut rand::thread_rng()),
                    p_dist.sample(&mut rand::thread_rng()),
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

    // Define timestep
    world.insert(Timestep { delta: 1.0e-5 });

    // Run the simulation for a number of steps.
    for _i in 0..10000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
