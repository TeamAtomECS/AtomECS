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
use lib::species::{Rubidium87_780D2};
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
use bevy::prelude::*;
use std::fs::read_to_string;

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

    let mut app = App::new();
    app.add_plugin(lib::integrator::IntegrationPlugin);
    app.add_plugin(lib::magnetic::MagneticsPlugin);
    app.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    app.add_plugin(LaserCoolingPlugin::<Rubidium87_780D2, {BEAM_NUMBER}>::default());
    app.add_system(lib::output::console_output::console_output);
    app.add_plugins(DefaultPlugins);
    app.add_system(lib::bevy_bridge::copy_positions);
    app.add_startup_system(setup_world);
    app.add_startup_system(create_atoms);
    app.add_startup_system(setup_camera);
    app.insert_resource(lib::bevy_bridge::Scale { 0: 2e3 });
    app.insert_resource(Timestep { delta: 5.0e-6 });
    app.insert_resource(EmissionForceOption::On(EmissionForceConfiguration {
        explicit_threshold: 5,
    }));
    app.insert_resource(ScatteringFluctuationsOption::On);

    app.run();
}

pub fn setup_world(mut commands: Commands) {

    //Load configuration if one exists.
    let read_result = read_to_string("doppler.json");
    let configuration: DopperSimulationConfiguration = match read_result {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap(),
        Err(_) => DopperSimulationConfiguration::default(),
    };

    // Create magnetic field.
    commands.spawn()
        .insert(QuadrupoleField3D::gauss_per_cm(0.001 * 18.2, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });

    // Create cooling lasers.
    let detuning = configuration.detuning;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    commands.spawn()
        .insert(GaussianBeam {
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
    commands.spawn()
        .insert(GaussianBeam {
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
    commands.spawn()
        .insert(GaussianBeam {
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
    commands.spawn()
        .insert(GaussianBeam {
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
    commands.spawn()
        .insert(GaussianBeam {
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
    commands.spawn()
        .insert(GaussianBeam {
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
}

fn create_atoms(mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let vel_dist = Normal::new(0.0, 0.22).unwrap();
    let pos_dist = Normal::new(0.0, 1.2e-4).unwrap();
    let mut rng = rand::thread_rng();

    // Add atoms
    for _ in 0..2000 {
        commands.spawn()
            .insert(Position {
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
            .insert(NewlyCreated)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(1.5, 0.5, 1.5),
                ..default()
            })
            ;
        }
    }

fn setup_camera(
    mut commands: Commands
) {
    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 3.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands.spawn_bundle(camera);


    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });
}