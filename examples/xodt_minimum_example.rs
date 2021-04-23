//! Loading a Sr cross beam dipole trap from center.
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom;
use lib::dipole;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::output::file::Text;
use lib::output::{file, xyz_file};
use nalgebra::Vector3;
use specs::{Builder, World};
use std::time::Instant;

fn main() {
    let now = Instant::now();

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<atom::Position, Text>("pos_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<atom::Velocity, Text>("vel_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(xyz_file::WriteToXYZFileSystem, "", &[]);

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    world
        .create_entity()
        .with(xyz_file::XYZWriteHelper {
            overwrite: true,
            initialized: false,
            write_every: 100,
            scale_factor: 20000.,
            discard_place: Vector3::new(2., 2., 2.),
            name: format!("{}", "xodt_min_example"),
        })
        .build();

    // Create dipole laser.

    let power = 10.0;
    let e_radius = 60.0e-6 / (2.0_f64.sqrt());

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::x(),
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::gaussian::GaussianReferenceFrame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
            ellipticity: 0.0,
        })
        .with(laser::gaussian::make_gaussian_rayleigh_range(
            &1064.0e-9,
            &gaussian_beam,
        ))
        .build();

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::y(),
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::gaussian::GaussianReferenceFrame {
            x_vector: Vector3::x(),
            y_vector: Vector3::z(),
            ellipticity: 0.0,
        })
        .with(laser::gaussian::make_gaussian_rayleigh_range(
            &1064.0e-9,
            &gaussian_beam,
        ))
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-5 });

    // Create a single test atom
    world
        .create_entity()
        .with(atom::Mass { value: 87.0 })
        .with(atom::Force::new())
        .with(atom::Position {
            pos: Vector3::new(-5.0e-6, 5.0e-6, 5.0e-6),
        })
        .with(atom::Velocity {
            vel: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(dipole::atom::AtomicDipoleTransition::strontium())
        .with(atom::Atom)
        .with(lib::initiate::NewlyCreated)
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..1_00_000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
