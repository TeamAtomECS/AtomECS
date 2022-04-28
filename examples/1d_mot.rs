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
use lib::species::{Strontium88_461};
use nalgebra::Vector3;
use bevy::prelude::*;

const BEAM_NUMBER : usize = 2;

fn main() {

    let mut app = App::new();
    app.add_plugin(lib::integrator::IntegrationPlugin);
    app.add_plugin(lib::magnetic::MagneticsPlugin);
    app.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    app.add_plugin(LaserCoolingPlugin::<Strontium88_461, {BEAM_NUMBER}>::default());
    app.add_system(lib::output::console_output::console_output);
    app.add_plugins(DefaultPlugins);
    app.add_system(lib::bevy_bridge::copy_positions);
    app.add_startup_system(setup_world);
    app.add_startup_system(create_atoms);
    app.add_startup_system(setup_camera);
    app.insert_resource(lib::bevy_bridge::Scale { 0: 1e2 });
    app.world.insert_resource(Timestep { delta: 1e-6 });
    app.run();
}

pub fn setup_world(mut commands: Commands) {

    // Create magnetic field.
    commands
        .spawn()
        .insert(QuadrupoleField3D::gauss_per_cm(15.0, Vector3::z()))
        .insert(Position::default());

    // Create cooling lasers.
    let detuning = -12.0;
    let power = 0.03;
    commands.spawn()
        .insert(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power,
            direction: -Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning,
            -1,
        ));
    commands.spawn()
        .insert(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning,
            -1,
        ));
}

fn create_atoms(mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in 0..100 {
        commands.spawn()
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, -0.05),
            })
            .insert(Atom)
            .insert(Force::default())
            .insert(Velocity {
                vel: Vector3::new(0.0, 0.0, 10.0 + (i as f64) * 5.0),
            })
            .insert(NewlyCreated)
            .insert(Strontium88_461)
            .insert(Mass { value: 87.0 })
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