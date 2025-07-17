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
use bevy::prelude::*;

const BEAM_NUMBER : usize = 6;

fn main() {

    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(LaserCoolingPlugin::<Strontium88_461, {BEAM_NUMBER}>::default());
    sim_builder.add_plugins(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(), 10));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world_mut()
        .spawn(QuadrupoleField3D::gauss_per_cm(15.0, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });


    // Create cooling lasers.
    let detuning = -12.0;
    let power = 0.03;
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power,
            direction: -Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning, -1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning, -1,
        ));

    // Create atoms
    for i in 0..20 {
        sim.world_mut()
            .spawn(Position {
                pos: Vector3::new(0.0,0.0, -0.05),
            })
            .insert(Velocity {
                vel: Vector3::new(0.0, 0.0, 10.0 + (i as f64) * 5.0),
            })
            .insert(Force::default())
            .insert(Mass { value: 87.0 })
            .insert(Strontium88_461)
            .insert(Atom)
            .insert(NewlyCreated);
    }
    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-6 });

    // Run the simulation for a number of steps.
    for _i in 0..5000 {
        sim.update();
    }
}
