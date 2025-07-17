//! An example of trapping atoms in a quadrupole trap.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::magnetic::force::MagneticDipole;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::simulation::SimulationBuilder;
use rand_distr::{Distribution, Normal};

use lib::output::file::FileOutputPlugin;
use lib::output::file::Text;
use nalgebra::Vector3;
use bevy::prelude::*;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(FileOutputPlugin::<Position, Text, Atom>::new(
        "pos.txt".to_string(),
        100,
    ));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new(
        "vel.txt".to_string(),
        100,
    ));

    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world_mut()
    .spawn((
        QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()),
        Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        },
    ));

    let p_dist = Normal::new(0.0, 0.5e-3).unwrap();
    let v_dist = Normal::new(0.0, 0.09).unwrap(); //80uK

    for _i in 0..1000 {
        sim.world_mut()
            .spawn((
                Position {
                    pos: Vector3::new(
                        p_dist.sample(&mut rand::rng()),
                        p_dist.sample(&mut rand::rng()),
                        p_dist.sample(&mut rand::rng()),
                    ),
                },
                Atom,
                Force::default(),
                Velocity {
                    vel: Vector3::new(
                        v_dist.sample(&mut rand::rng()),
                        v_dist.sample(&mut rand::rng()),
                        v_dist.sample(&mut rand::rng()),
                    ),
                },
                NewlyCreated,
                MagneticDipole { mFgF: 0.5 },
                Mass { value: 87.0 },
            ));
    }

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-5 });

    // Run the simulation for a number of steps.
    for _i in 0..10000 {
        sim.update();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
