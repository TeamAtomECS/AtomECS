//! Time-Orbiting Potential (TOP) trap
//! 
//! cargo build --example top_trap --target wasm32-unknown-unknown
//! wasm-bindgen --out-dir OUTPUT_DIR --target web TARGET_DIR
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::magnetic::force::{MagneticDipole};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::magnetic::top::UniformFieldRotator;
//use lib::simulation::SimulationBuilder;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};
use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    // Add magnetics systems (todo: as plugin)
    app.add_plugin(lib::integrator::IntegrationPlugin);
    app.add_plugin(lib::magnetic::MagneticsPlugin);
    app.add_system(lib::output::console_output::console_output);
    app.add_plugins(DefaultPlugins);
    app.add_system(lib::bevy_bridge::copy_positions);
    app.add_startup_system(setup);
    app.insert_resource(lib::bevy_bridge::Scale { 0: 1e4 });
    app.add_startup_system(setup_atoms);

    // Create magnetic field.
    app.world.spawn()
        .insert(QuadrupoleField3D::gauss_per_cm(80.0, Vector3::z()))
        .insert(Position::default());

    app.world.spawn()
        .insert(UniformFieldRotator { amplitude: 20.0, frequency: 3000.0 }) // Time averaged TOP theory assumes rotation frequency much greater than velocity of atoms
        .insert(lib::magnetic::uniform::UniformMagneticField { field: Vector3::new(0.0,0.0,0.0)}) // Time averaged TOP theory assumes rotation frequency much greater than velocity of atoms
        ;

    // Define timestep
    app.world.insert_resource(Timestep { delta: 5e-5 }); //Aliasing of TOP field or other strange effects can occur if timestep is not much smaller than TOP field period.
                                                //Timestep must also be much smaller than mean collision time.

    // Run the simulation for a number of steps.
    // for _i in 0..10000 {
    //      app.update();
    // }
    app.run();
}

fn setup_atoms(mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let p_dist = Normal::new(0.0, 50e-6).unwrap();
    let v_dist = Normal::new(0.0, 0.004).unwrap(); // ~100nK

    for _i in 0..5000 {
        commands
            .spawn()
            .insert(Position {
                pos: Vector3::new(
                    p_dist.sample(&mut rand::thread_rng()),
                    p_dist.sample(&mut rand::thread_rng()),
                    0.35 * p_dist.sample(&mut rand::thread_rng()), //TOP traps have tighter confinement along quadrupole axis
                ),
            })
            .insert(Atom)
            .insert(Force::default())
            .insert(Velocity {
                vel: Vector3::new(
                    v_dist.sample(&mut rand::thread_rng()),
                    v_dist.sample(&mut rand::thread_rng()),
                    v_dist.sample(&mut rand::thread_rng()),
                ),
            })
            .insert(NewlyCreated)
            .insert(MagneticDipole { mFgF: 0.5 })
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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