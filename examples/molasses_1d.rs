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
use bevy::prelude::*;

const BEAM_NUMBER : usize = 2;

fn main() {
    
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(LaserCoolingPlugin::<Rubidium87_780D2, {BEAM_NUMBER}>::default());
    sim_builder.add_plugins(FileOutputPlugin::<ActualPhotonsScatteredVector<Rubidium87_780D2, {BEAM_NUMBER}>, Text, Atom>::new("scattered.txt".to_string(), 10));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Create atoms
    for i in 0..20 {
        sim.world_mut()
            .spawn((
                Position {
                    pos: Vector3::new(0.0, 0.0, -0.03),
                },
                Atom,
                Force::default(),
                Velocity {
                    vel: Vector3::new(0.0, 0.0, 10.0 + (i as f64) * 1.0),
                },
                NewlyCreated,
                Rubidium87_780D2,
                Mass { value: 87.0 },
            ));
    }

    // Create cooling lasers.
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 0.01,
            direction: -Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            -6.0,
            -1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 0.01,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            -6.0,
            -1,
        ));

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-6 });

    // Run the simulation for a number of steps.
    for _i in 0..1600 {
        sim.update();
    }
}
