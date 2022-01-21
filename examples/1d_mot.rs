//! Simulate a 1D MOT.
//!
//! The 1D MOT is formed by counter-propagating laser beams along the z-axis.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::LaserPlugin;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use lib::species::{Strontium88_461};
use nalgebra::Vector3;
use specs::prelude::*;

const BEAM_NUMBER : usize = 6;

fn main() {

    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugin(LaserCoolingPlugin::<Strontium88_461, {BEAM_NUMBER}>::default());
    sim_builder.add_plugin(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(), 10));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(15.0, Vector3::z()))
        .with(Position::new())
        .build();

    // Create cooling lasers.
    let detuning = -12.0;
    let power = 0.03;
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power,
            direction: -Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Strontium88_461>(
            detuning,
            -1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Strontium88_461>(
            detuning,
            -1,
        ))
        .build();

    // Create atoms
    for i in 0..20 {
        sim.world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, -0.05),
            })
            .with(Atom)
            .with(Force::new())
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 10.0 + (i as f64) * 5.0),
            })
            .with(NewlyCreated)
            .with(Strontium88_461)
            .with(Mass { value: 87.0 })
            .build();
    }
    // Define timestep
    sim.world.insert(Timestep { delta: 1.0e-6 });

    // Run the simulation for a number of steps.
    for _i in 0..5000 {
        sim.step();
    }
}
