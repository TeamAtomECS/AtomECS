//! Loading a Sr cross beam dipole trap from center.
use atomecs::collisions::ApplyCollisionsOption;
use atomecs::collisions::CollisionParameters;
use atomecs::collisions::CollisionsTracker;
use specs::prelude::*;
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::Atom;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::central_creator::CentralCreator;
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::destructor::ToBeDestroyed;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::output::file;
use lib::output::file::{Text, XYZ};
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
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
        file::new::<Position, Text>("pos_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new_with_filter::<Position, XYZ, Atom>("position.xyz".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    // Create dipole laser.
    let power = 20.0;
    let e_radius = 20.0e-6 / (2.0_f64.sqrt());

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::x(),
        rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(&1064.0e-9, &e_radius),
        ellipticity: 0.0,
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(laser::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::frame::Frame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
        })
        .build();

    // creating the entity that represents the source
    //
    // contains a central creator
    let number_to_emit = 1_000;
    let size_of_cube = 1.0e-5;
    let speed = 0.005; // m/s

    world
        .create_entity()
        .with(CentralCreator::new_uniform_cubic(size_of_cube, speed))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 87.0,
            ratio: 1.0,
        }]))
        .with(AtomicTransition::strontium_red())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();

    // Define timestep
    world.insert(Timestep { delta: 1.0e-6 });
    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.0002, 0.0002, 0.0002),
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    world.insert(ApplyCollisionsOption);

    world.insert(CollisionParameters {
        macroparticle: 10000.0,
        box_number: 1000000,
        box_width: 5.0e-6,
        sigma: 4.0 * lib::constant::PI * (96.0 * 5.291e-11 as f64).powi(2),
        collision_limit: 10_000_000.0,
    });
    world.insert(CollisionsTracker {
        num_collisions: Vec::new(),
        num_atoms: Vec::new(),
        num_particles: Vec::new(),
    });

    let mut switcher_system =
        atomecs::dipole::transition_switcher::AttachAtomicDipoleTransitionToAtomsSystem;

    dispatcher.dispatch(&mut world);
    world.maintain();
    switcher_system.run_now(&world);

    // Run the simulation for a number of steps.
    for _i in 0..50_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
