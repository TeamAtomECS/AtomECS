//! Single particle in a cross beam optical dipole trap
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{self, Position, Velocity};
use lib::atom::Atom;
use lib::dipole::{self, DipolePlugin};
use lib::integrator::Timestep;
use lib::laser::{self, LaserPlugin};
use lib::laser::gaussian::GaussianBeam;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::{Text, XYZ};
use lib::simulation::SimulationBuilder;
use nalgebra::Vector3;
use specs::prelude::*;
use std::time::Instant;

const BEAM_NUMBER: usize = 1;

fn main() {
    let now = Instant::now();

    // Configure simulation output.
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugin(DipolePlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugin(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(), 1));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 1));
    sim_builder.add_plugin(FileOutputPlugin::<Position, XYZ, Atom>::new("position.xyz".to_string(), 1));
    let mut sim = sim_builder.build();

    // Create dipole laser.
    let power = 100.0;
    let e_radius = 60.0e-6 / 2.0_f64.sqrt();
    let wavelength = 1064.0e-9;

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius,
        power,
        direction: Vector3::z(),
        rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        ellipticity: 0.0,
    };
    sim.world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::DipoleLight { wavelength })
        .with(laser::frame::Frame {
            x_vector: Vector3::x(),
            y_vector: Vector3::y(),
        })
        .build();

    // Create a single test atom
    sim.world
        .create_entity()
        .with(atom::Mass { value: 87.0 })
        .with(atom::Force::new())
        .with(atom::Position {
            pos: Vector3::new(1.0e-7, 1.0e-7, 1.0e-7),
        })
        .with(atom::Velocity {
            vel: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(dipole::Polarizability::calculate_for(
            wavelength, 461e-9, 32.0e6,
        ))
        .with(atom::Atom)
        .with(lib::initiate::NewlyCreated)
        .build();

    // Define timestep
    sim.world.insert(Timestep { delta: 1.0e-7 });

    // Run the simulation for a number of steps.
    for _i in 0..200_000 {
        sim.step();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
