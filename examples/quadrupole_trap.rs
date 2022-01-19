//! An example of trapping atoms in a quadrupole trap.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::magnetic::force::{ApplyMagneticForceSystem, MagneticDipole};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::simulation::SimulationBuilder;
use lib::species::{Rubidium87_780D2, Rubidium87};
use rand_distr::{Distribution, Normal};

use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use nalgebra::Vector3;
use specs::prelude::*;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    let mut sim_builder = SimulationBuilder::default::<Rubidium87_780D2, Rubidium87>();
    sim_builder.add_plugin(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(), 100));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 100));
    
    // Add magnetics systems (todo: as plugin)
    sim_builder.world.register::<NewlyCreated>();
    sim_builder.world.register::<MagneticDipole>();
    sim_builder.dispatcher_builder.add(
        ApplyMagneticForceSystem {},
        "magnetic_force",
        &["magnetics_gradient"],
    );
    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()))
        .with(Position::new())
        .build();

    let p_dist = Normal::new(0.0, 0.5e-3).unwrap();
    let v_dist = Normal::new(0.0, 0.09).unwrap(); //80uK

    for _i in 0..1000 {
        sim.world
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
    sim.world.insert(Timestep { delta: 1.0e-5 });

    // Run the simulation for a number of steps.
    for _i in 0..10000 {
        sim.step();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
