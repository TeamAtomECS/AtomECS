//! Loading a Sr 3D MOT directly from an oven source.

extern crate atomecs as lib;
extern crate nalgebra;
use atomecs::atom::Atom;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::atom_sources::oven::{OvenAperture, OvenBuilder};
use lib::destructor::ToBeDestroyed;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::force::EmissionForceOption;
use lib::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
use specs::prelude::*;
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
        file::new::<Position, Text, Atom>("pos_animation1.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text, Atom>("vel_animation1.txt".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()))
        .with(Position::new())
        .build();

    // Create cooling lasers.
    let detuning = -15.5;
    let power = 10.0;
    let radius = 40.0e-3 / (2.0 * 2.0_f64.sqrt()); // 33mm 1/e^2 diameter

    // Horizontal beams along z
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: -Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            -1,
        ))
        .build();

    // Angled vertical beams
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, -1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, 1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            1,
        ))
        .build();

    // Create an oven.
    // The oven will eject atoms on the first frame and then be deleted.
    let number_to_emit = 3_200_000;
    world
        .create_entity()
        .with(
            OvenBuilder::new(200.0, Vector3::x())
                .with_aperture(OvenAperture::Circular {
                    radius: 0.005,
                    thickness: 0.001,
                })
                .build(),
        )
        .with(Position {
            pos: Vector3::new(-0.083, 0.0, 0.0),
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 88.0,
            ratio: 1.0,
        }]))
        .with(AtomicTransition::strontium())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();

    // Define timestep
    world.insert(Timestep { delta: 1.0e-6 });
    // enable the usage of the emission system
    world.insert(EmissionForceOption::default());
    //enable gravity
    world.insert(lib::gravity::ApplyGravityOption);
    //enable the fluctuations in Photon numbers
    world.insert(ScatteringFluctuationsOption::default());

    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.1, 0.01, 0.01), //(0.1, 0.01, 0.01)
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..100_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
