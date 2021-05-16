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
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::force::{EmissionForceConfiguration, EmissionForceOption};
use lib::laser::gaussian::GaussianBeam;
use lib::laser::photons_scattered::ScatteringFluctuationsOption;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use rand::distributions::{Distribution, Normal};
use specs::{Builder, World};
use std::fs::read_to_string;
use std::time::Instant;

extern crate serde;
use serde::Deserialize;

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

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 10),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(0.001 * 18.2, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    // Create cooling lasers.
    let detuning = configuration.detuning;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 0.0, 1.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 0.0, -1.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, 0.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 1.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, -1.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    let vel_dist = Normal::new(0.0, 0.22);
    let pos_dist = Normal::new(0.0, 1.2e-4);
    let mut rng = rand::thread_rng();

    // Add atoms
    for _ in 0..2000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng),
                ),
            })
            .with(Velocity {
                vel: Vector3::new(
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng),
                ),
            })
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(AtomicTransition::rubidium())
            .with(Atom)
            .with(NewlyCreated)
            .build();
    }

    // Enable fluctuation options
    //  * Allow photon numbers to fluctuate.
    //  * Allow random force from emission of photons.
    world.add_resource(EmissionForceOption::On(EmissionForceConfiguration {
        explicit_threshold: 5,
    }));
    world.add_resource(ScatteringFluctuationsOption::On);

    // Run the simulation for a number of steps.
    for _i in 0..configuration.number_of_steps {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
