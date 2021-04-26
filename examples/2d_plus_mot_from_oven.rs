//! A 2D+ mot configuration, loaded directly from oven.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::atom_sources::oven::{OvenAperture, OvenBuilder};
use lib::atom_sources::VelocityCap;
use lib::constant;
use lib::destructor::ToBeDestroyed;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
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
        file::new::<Position, Text>("pos.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()))
        .with(Position::new())
        .build();

    // Push beam along z
    let push_beam_radius = 1e-3;
    let push_beam_power = 0.010;
    let push_beam_detuning = 0.0;

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: push_beam_radius,
            power: push_beam_power,
            direction: Vector3::z(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &push_beam_radius,
            ),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            push_beam_detuning,
            -1,
        ))
        .build();

    // Create cooling lasers.
    let detuning = -45.0;
    let power = 0.23;
    let radius = 33.0e-3 / (2.0 * 2.0_f64.sqrt()); // 33mm 1/e^2 diameter
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            rayleigh_range: lib::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            1,
        ))
        .build();

    // Create an oven.
    // The oven will eject atoms on the first frame and then be deleted.
    let number_to_emit = 400000;
    world
        .create_entity()
        .with(
            OvenBuilder::new(776.0, Vector3::x())
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
    world.add_resource(Timestep { delta: 1.0e-6 });

    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation.
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.1, 0.01, 0.01),
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // The simulation bound also now includes a small pipe to capture the 2D MOT output properly.
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.1),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.01, 0.01, 0.1),
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // Also use a velocity cap so that fast atoms are not even simulated.
    world.add_resource(VelocityCap { value: 200.0 });

    // Run the simulation for a number of steps.
    for _i in 0..10000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
