//! Single particle in a cross beam optical dipole trap
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{self, Position, Velocity};
use lib::atom::Atom;
use lib::dipole::{self, DipolePlugin};
use lib::integrator::Timestep;
use lib::laser::{self, LaserPlugin, RequiresIntensityGradientCalculation};
use lib::laser::gaussian::GaussianBeam;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::{Text, XYZ};
use lib::simulation::SimulationBuilder;
use nalgebra::Vector3;
use bevy::prelude::*;
use std::time::Instant;

const BEAM_NUMBER: usize = 2;

fn main() {
    let now = Instant::now();

    // Configure simulation output.
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(DipolePlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(), 100));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 100));

    // I am unclear on whether to implement XYZPosition for Position or whether the intent was to deprecate XYZ format. 
    // Commenting out for the time being.
    // sim_builder.add_plugins(FileOutputPlugin::<Position, XYZ, Atom>::new("position.xyz".to_string(), 100));
    let mut sim = sim_builder.build();

    // Create dipole laser.
    let power = 10.0;
    let e_radius = 60.0e-6 / (2.0_f64.sqrt());
    let wavelength = 1064.0e-9;

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius,
        power,
        direction: Vector3::x(),
        rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        ellipticity: 0.0,
    };
    sim.world_mut()
        .spawn(gaussian_beam)
        .insert(dipole::DipoleLight { wavelength })
        .insert(laser::frame::Frame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
        })
        .insert(RequiresIntensityGradientCalculation);

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius,
        power,
        direction: Vector3::y(),
        rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        ellipticity: 0.0,
    };
    sim.world_mut()
        .spawn(gaussian_beam)
        .insert(dipole::DipoleLight { wavelength })
        .insert(laser::frame::Frame {
            x_vector: Vector3::x(),
            y_vector: Vector3::z(),
        })
        .insert(RequiresIntensityGradientCalculation);

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-5 });

    // Create a single test atom
    sim.world_mut()
        .spawn((
            atom::Mass { value: 87.0 },
            atom::Force::default(),
            atom::Position {
                pos: Vector3::new(-5.0e-6, 5.0e-6, 5.0e-6),
            },
            atom::Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            },
            dipole::Polarizability::calculate_for(
                wavelength, 461e-9, 32.0e6,
            ),
            atom::Atom,
            lib::initiate::NewlyCreated,
        ));

    // Run the simulation for a number of steps.
    for _i in 0..100_000 {
        sim.update();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
