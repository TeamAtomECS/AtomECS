//! # Doppler Sweep
//!
//! Simulate a cloud of atoms in a 3D MOT to measure the Doppler temperature limit for laser cooling.
//!
//! The Doppler Limit depends on temperature, see eg https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.61.169.
//!
//! Some parameters of the simulation can be set by writing a configuration file called `doppler.json`. This file
//! allows the user to control parameters, eg detuning. If the file is not written, a default detuning of 0.5 Gamma
//! is used, which corresponds to the minimum Doppler temperature.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::LaserPlugin;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::force::{EmissionForceConfiguration, EmissionForceOption};
use lib::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use lib::species::{Rubidium87_780D2};
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
use bevy::prelude::*;
use std::fs::read_to_string;
use std::time::Instant;

extern crate serde;
use serde::Deserialize;

const BEAM_NUMBER : usize = 6;

#[derive(Deserialize)]
pub struct DopperSimulationConfiguration {
    /// Detuning of laser beams, in units of MHz.
    pub detuning: f64,
    /// Number of simulation steps to evolve for.
    pub number_of_steps: i32,
}
impl Default for DopperSimulationConfiguration {
    fn default() -> Self {
        DopperSimulationConfiguration {
            detuning: -3.0,
            number_of_steps: 5000,
        }
    }
}

fn main() {
    let now = Instant::now();

    //Load configuration if one exists.
    let read_result = read_to_string("doppler.json");
    let configuration: DopperSimulationConfiguration = match read_result {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap(),
        Err(_) => DopperSimulationConfiguration::default(),
    };

    // Create the simulation
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(LaserCoolingPlugin::<Rubidium87_780D2, {BEAM_NUMBER}>::default());
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world_mut()
        .spawn(QuadrupoleField3D::gauss_per_cm(0.001 * 18.2, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });

    // Create cooling lasers.
    let detuning = configuration.detuning;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, 1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            -1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, -1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            -1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, -1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(   
            detuning,
            1,
        ));

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-6 });

    let vel_dist = Normal::new(0.0, 0.22).unwrap();
    let pos_dist = Normal::new(0.0, 1.2e-4).unwrap();
    let mut rng = rand::rng();

    // Add atoms
    for _ in 0..2000 {
        sim.world_mut()
            .spawn((
                Position {
                    pos: Vector3::new(
                        pos_dist.sample(&mut rng),
                        pos_dist.sample(&mut rng),
                        pos_dist.sample(&mut rng),
                    ),
                },
                Velocity {
                    vel: Vector3::new(
                        vel_dist.sample(&mut rng),
                        vel_dist.sample(&mut rng),
                        vel_dist.sample(&mut rng),
                    ),
                },
                Force::default(),
                Mass { value: 87.0 },
                Rubidium87_780D2,
                Atom,
                NewlyCreated,
            ));
    }

    // Enable fluctuation options
    //  * Allow photon numbers to fluctuate.
    //  * Allow random force from emission of photons.
    sim.world_mut().insert_resource(EmissionForceOption::On(EmissionForceConfiguration {
        explicit_threshold: 5,
    }));
    sim.world_mut().insert_resource(ScatteringFluctuationsOption::On);

    // Run the simulation for a number of steps.
    for _i in 0..configuration.number_of_steps {
        sim.update();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
