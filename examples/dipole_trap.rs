//! Loading a Sr cross beam dipole trap from center.
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom;
use lib::atom::Atom;
use lib::dipole;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::output::file;
use lib::output::file::{Text, XYZ};
use nalgebra::Vector3;
use specs::prelude::*;
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
        file::new::<atom::Position, Text>("pos_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<atom::Velocity, Text>("vel_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new_with_filter::<atom::Position, XYZ, Atom>("position.xyz".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);
    // Create dipole laser.

    let power = 7.0;
    let e_radius = 50.0e-6 / (2.0_f64.sqrt());
    let wavelength = 1064.0e-9;

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::x(),
        rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        ellipticity: 0.0,
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(laser::dipole_beam::DipoleLight {
            wavelength: wavelength,
        })
        .with(laser::frame::Frame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
        })
        .build();

    // Define timestep
    world.insert(Timestep { delta: 1.0e-7 });

    // Create a single test atom
    world
        .create_entity()
        .with(atom::Mass { value: 87.0 })
        .with(atom::Force::new())
        .with(atom::Position {
            pos: Vector3::new(-0.0e-6, 1.0e-6, 0.0e-6),
        })
        .with(atom::Velocity {
            vel: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(dipole::atom::AtomicDipoleTransition::strontium())
        .with(atom::Atom)
        .with(lib::initiate::NewlyCreated)
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..100_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
