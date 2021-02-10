//! Simulation of atoms cooled to the Doppler limit.

extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::force::ApplyEmissionForceOption;
use lib::laser::gaussian::GaussianBeam;
use lib::laser::photons_scattered::EnableScatteringFluctuations;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use nalgebra::Vector3;
use rand::distributions::{Distribution, Normal};
use specs::{Builder, World};
use std::fs::read_to_string;
use std::time::Instant;

extern crate serde;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BenchmarkConfiguration {
    pub n_threads: usize,
    pub n_atoms: i32,
    pub n_steps: i32,
}
impl Default for BenchmarkConfiguration {
    fn default() -> Self {
        BenchmarkConfiguration {
            n_atoms: 10000,
            n_threads: 12,
            n_steps: 5000,
        }
    }
}

fn main() {
    let now = Instant::now();

    //Load configuration if one exists.
    let read_result = read_to_string("benchmark.json");
    let configuration: BenchmarkConfiguration = match read_result {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap(),
        Err(_) => BenchmarkConfiguration::default(),
    };

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure thread pool.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(configuration.n_threads)
        .build()
        .unwrap();

    builder.add_pool(::std::sync::Arc::new(pool));

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(18.2, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    // Create cooling lasers.
    let detuning = -3.0;
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
    for _ in 0..configuration.n_atoms {
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
    world.add_resource(ApplyEmissionForceOption {});
    world.add_resource(EnableScatteringFluctuations {});

    // Run the simulation for a number of steps.
    for _i in 0..configuration.n_steps {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
