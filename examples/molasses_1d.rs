extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::LaserPlugin;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::photons_scattered::ActualPhotonsScatteredVector;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use lib::simulation::{SimulationBuilder};
use lib::species::{Rubidium87_780D2};
use nalgebra::Vector3;
use specs::prelude::*;

const BEAM_NUMBER : usize = 2;

fn main() {
    
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugin(LaserCoolingPlugin::<Rubidium87_780D2, {BEAM_NUMBER}>::default());
    sim_builder.add_plugin(FileOutputPlugin::<ActualPhotonsScatteredVector<Rubidium87_780D2, {BEAM_NUMBER}>, Text, Atom>::new("scattered.txt".to_string(), 10));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Create atoms
    for i in 0..20 {
        sim.world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, -0.03),
            })
            .with(Atom)
            .with(Force::new())
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 10.0 + (i as f64) * 1.0),
            })
            .with(NewlyCreated)
            .with(Rubidium87_780D2)
            .with(Mass { value: 87.0 })
            .build();
    }

    // Create cooling lasers.
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 0.01,
            direction: -Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            -6.0,
            -1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 0.01,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            -6.0,
            -1,
        ))
        .build();

    // Define timestep
    sim.world.insert(Timestep { delta: 1.0e-6 });

    // Run the simulation for a number of steps.
    for _i in 0..1600 {
        sim.step();
    }
}
