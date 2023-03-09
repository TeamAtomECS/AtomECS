//! Benchmark simulation as a reference for AtomECS performance.

extern crate atomecs as lib;
extern crate nalgebra;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::gaussian::GaussianBeam;
use lib::laser::LaserPlugin;
use lib::laser_cooling::force::EmissionForceOption;
use lib::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::species::Rubidium87_780D2;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
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

const BEAM_NUMBER: usize = 6;

fn main() {
    //Load configuration if one exists.
    let read_result = read_to_string("benchmark.json");
    let configuration: BenchmarkConfiguration = match read_result {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap(),
        Err(_) => BenchmarkConfiguration::default(),
    };

    let mut app = App::new();
    app.add_plugin(LaserPlugin::<{ BEAM_NUMBER }>);
    app.add_plugin(LaserCoolingPlugin::<Rubidium87_780D2, { BEAM_NUMBER }>::default());
    app.add_plugins(
        DefaultPlugins.set(TaskPoolPlugin {
            task_pool_options: TaskPoolOptions::with_num_threads(10),
        }), // .set(LogPlugin {
            //     level: Level::DEBUG,
            //     filter: "bevy_core=trace".to_string(),
            // }),
    );
    app.add_plugin(atomecs::integrator::IntegrationPlugin);
    app.add_plugin(atomecs::initiate::InitiatePlugin);
    app.add_plugin(atomecs::magnetic::MagneticsPlugin);
    app.add_plugin(atomecs::sim_region::SimulationRegionPlugin);
    app.add_system(atomecs::output::console_output::console_output);
    //app.add_startup_system(setup_world);

    // TODO: Configure bevy compute pool size

    // Create magnetic field.
    app.world
        .spawn(QuadrupoleField3D::gauss_per_cm(18.2, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });

    // Create cooling lasers.
    let detuning = -3.0;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    app.world
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, 1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, -1,
        ));
    app.world
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, -1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, -1,
        ));
    app.world
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
    app.world
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
    app.world
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
    app.world
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, -1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));

    // Define timestep
    app.world.insert_resource(Timestep { delta: 1.0e-6 });

    let vel_dist = Normal::new(0.0, 0.22).unwrap();
    let pos_dist = Normal::new(0.0, 1.2e-4).unwrap();
    let mut rng = rand::thread_rng();

    // Add atoms
    for _ in 0..configuration.n_atoms {
        app.world
            .spawn(Position {
                pos: Vector3::new(
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng),
                ),
            })
            .insert(Velocity {
                vel: Vector3::new(
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng),
                ),
            })
            .insert(Force::default())
            .insert(Mass { value: 87.0 })
            .insert(Rubidium87_780D2)
            .insert(Atom)
            .insert(NewlyCreated);
    }

    // Enable fluctuation options
    //  * Allow photon numbers to fluctuate.
    //  * Allow random force from emission of photons.
    app.world.insert_resource(EmissionForceOption::default());
    app.world
        .insert_resource(ScatteringFluctuationsOption::default());

    let loop_start = Instant::now();

    // Run the simulation for a number of steps.
    for _i in 0..configuration.n_steps {
        app.update();
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
