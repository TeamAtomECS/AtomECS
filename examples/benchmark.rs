//! Benchmark simulation as a reference for AtomECS performance.

extern crate atomecs as lib;
extern crate nalgebra;

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

use bevy::app::TaskPoolThreadAssignmentPolicy;

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
            n_atoms: 1_000_000,
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
    app.add_plugins(LaserPlugin::<{ BEAM_NUMBER }>);
    app.add_plugins(LaserCoolingPlugin::<Rubidium87_780D2, { BEAM_NUMBER }>::default());
    app.add_plugins(atomecs::integrator::IntegrationPlugin);
    app.add_plugins(atomecs::initiate::InitiatePlugin);
    app.add_plugins(atomecs::magnetic::MagneticsPlugin);
    app.add_plugins(atomecs::sim_region::SimulationRegionPlugin);
    app.add_systems(Update, atomecs::output::console_output::console_output);
    //app.add_startup_system(setup_world);

    // TODO: Configure bevy compute pool size
    
    // Enabling the task pool plugin with custom thread assignment policy settings,
    // requires the regex features to be enabled in Cargo.toml. This seems to perform the same as the default settings.
    let task_pool_options = TaskPoolOptions {
        // Use 25% of cores for IO, at least 1, no more than 4
        io: TaskPoolThreadAssignmentPolicy {
            min_threads: 0,
            max_threads: 0,
            percent: 0.0,
            on_thread_spawn: None,
            on_thread_destroy: None,
        },

        // Use 25% of cores for async compute, at least 1, no more than 4
        async_compute: TaskPoolThreadAssignmentPolicy {
            min_threads: 0,
            max_threads: 0,
            percent: 0.0,
            on_thread_spawn: None,
            on_thread_destroy: None,
        },
        min_total_threads: 1,
        max_total_threads: usize::MAX,
        compute: TaskPoolThreadAssignmentPolicy {
            min_threads: 1,
            max_threads: usize::MAX,
            percent: 100.0,
            on_thread_spawn: None,
            on_thread_destroy: None,
        },
    };
    
    app.add_plugins(
        DefaultPlugins
        .set(TaskPoolPlugin {
            task_pool_options
        }),
        // .set(LogPlugin {
        //     level: Level::DEBUG,
        //     filter: "bevy_core=trace".to_string(),
        // }),
    );

    // This (original) however seems to be worse
    // app.add_plugins(
    //     DefaultPlugins
    //     .set(TaskPoolPlugin {
    //         task_pool_options: TaskPoolOptions::with_num_threads(10),
    //     }),
    //     // .set(LogPlugin {
    //     //     level: Level::DEBUG,
    //     //     filter: "bevy_core=trace".to_string(),
    //     // }),
    // );

    // Create magnetic field.
    app.world_mut()
        .spawn(QuadrupoleField3D::gauss_per_cm(18.2, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });

    // Create cooling lasers.
    let detuning = -3.0;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    app.world_mut()
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
    app.world_mut()
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
    app.world_mut()
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
    app.world_mut()
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
    app.world_mut()
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
    app.world_mut()
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
    app.world_mut().insert_resource(Timestep { delta: 1.0e-6 });

    let vel_dist = Normal::new(0.0, 0.22).unwrap();
    let pos_dist = Normal::new(0.0, 1.2e-4).unwrap();
    let mut rng = rand::rng();

    // Add atoms
    for _ in 0..configuration.n_atoms {
        app.world_mut()
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
    app.world_mut().insert_resource(EmissionForceOption::default());
    app.world_mut()
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
