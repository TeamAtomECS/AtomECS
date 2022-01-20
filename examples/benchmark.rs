//! Simulation of atoms cooled to the Doppler limit.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::LaserPlugin;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::force::EmissionForceOption;
use lib::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::simulation::SimulationBuilder;
use lib::species::{Rubidium87_780D2};
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
use specs::prelude::*;
use std::fs::read_to_string;
use std::fs::File;
use std::time::Instant;

extern crate serde;
use serde::{Deserialize, Serialize};

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
#[derive(Serialize)]
pub struct SimulationOutput {
    pub time: f64,
}

const BEAM_NUMBER : usize = 6;

fn main() {
    //Load configuration if one exists.
    let read_result = read_to_string("benchmark.json");
    let configuration: BenchmarkConfiguration = match read_result {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap(),
        Err(_) => BenchmarkConfiguration::default(),
    };

    // Create the simulation world and builder for the ECS dispatcher.
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugin(LaserCoolingPlugin::<Rubidium87_780D2, {BEAM_NUMBER}>::default());

    // Configure thread pool.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(configuration.n_threads)
        .build()
        .unwrap();

    sim_builder.dispatcher_builder.add_pool(::std::sync::Arc::new(pool));
    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world
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

    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, 1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            -1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, -1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            -1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ))
        .build();
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, -1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ))
        .build();

    // Define timestep
    sim.world.insert(Timestep { delta: 1.0e-6 });

    let vel_dist = Normal::new(0.0, 0.22).unwrap();
    let pos_dist = Normal::new(0.0, 1.2e-4).unwrap();
    let mut rng = rand::thread_rng();

    // Add atoms
    for _ in 0..configuration.n_atoms {
        sim.world
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
            .with(Rubidium87_780D2)
            .with(Atom)
            .with(NewlyCreated)
            .build();
    }

    // Enable fluctuation options
    //  * Allow photon numbers to fluctuate.
    //  * Allow random force from emission of photons.
    sim.world.insert(EmissionForceOption::default());
    sim.world.insert(ScatteringFluctuationsOption::default());

    let loop_start = Instant::now();

    // Run the simulation for a number of steps.
    for _i in 0..configuration.n_steps {
        sim.step();
    }

    println!(
        "Simulation loop completed in {} ms.",
        loop_start.elapsed().as_millis()
    );

    serde_json::to_writer(
        File::create("benchmark_result.txt").expect("Could not open output file."),
        &SimulationOutput {
            time: loop_start.elapsed().as_secs_f64(),
        },
    )
    .expect("Could not write output file.");
}
