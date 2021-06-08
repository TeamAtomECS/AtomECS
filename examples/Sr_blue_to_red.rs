//! Loading a Sr cross beam dipole trap from center.

extern crate atomecs as lib;
extern crate nalgebra;
use crate::lib::laser_cooling::force::EmissionForceOption;
use atomecs::integrator::INTEGRATE_VELOCITY_SYSTEM_NAME;
use atomecs::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use atomecs::output::file::SerdeJson;
use atomecs::ramp::Ramp;
use lib::atom::Atom;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::central_creator::CentralCreator;
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::constant;
use lib::destructor::ToBeDestroyed;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::{Text, XYZ};
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
        file::new::<Position, Text>("pos.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new_with_filter::<Position, XYZ, Atom>("position.xyz".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new_with_filter::<lib::atom::Force, SerdeJson, Atom>("force.txt".to_string(), 100),
        "",
        &[INTEGRATE_VELOCITY_SYSTEM_NAME],
    );
    builder = builder.with(
        file::new_with_filter::<
            lib::laser_cooling::photons_scattered::TotalPhotonsScattered,
            SerdeJson,
            Atom,
        >("photons.txt".to_string(), 100),
        "",
        &[INTEGRATE_VELOCITY_SYSTEM_NAME],
    );

    builder = builder.with(
        file::new_with_filter::<QuadrupoleField3D, SerdeJson, QuadrupoleField3D>(
            "magnetic.txt".to_string(),
            100,
        ),
        "",
        &[INTEGRATE_VELOCITY_SYSTEM_NAME],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);
    // BEGIN MOT PART

    world.register::<Ramp<QuadrupoleField3D>>();

    let mut magnetic_frames = Vec::new();
    magnetic_frames.push((0.0, QuadrupoleField3D::gauss_per_cm(60.0, Vector3::x())));
    magnetic_frames.push((0.01, QuadrupoleField3D::gauss_per_cm(60.0, Vector3::x())));
    magnetic_frames.push((0.02, QuadrupoleField3D::gauss_per_cm(1.0, Vector3::x())));
    let magnetic_ramp = Ramp::new(magnetic_frames);

    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(60.0, Vector3::x()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0e-6),
        })
        .with(magnetic_ramp)
        .build();

    let detuning = -30.0; //MHz
    let power = 0.024; //W total power of all Lasers together
    let radius = 5.0e-3 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    // BLUE MOT
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            power: power / 6.0,
            direction: -Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
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
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium(),
            detuning,
            1,
        ))
        .build();
    world.insert(EmissionForceOption::default());
    world.insert(ScatteringFluctuationsOption::default());

    // RED MOT
    let detuning = -0.05; //MHz
    let power = 0.001; //W total power of all Lasers together
    let radius = 3.0e-3 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: -Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
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
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world.insert(EmissionForceOption::default());
    world.insert(ScatteringFluctuationsOption::default());

    // END MOT part

    // creating the entity that represents the source
    //
    // contains a central creator
    let number_to_emit = 1_000;
    let size_of_cube = 1.0e-3;
    let speed = 0.5; // m/s

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
        .with(AtomicTransition::strontium())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();

    // Define timestep
    world.insert(Timestep { delta: 1.0e-6 });

    //enable gravity
    world.insert(lib::gravity::ApplyGravityOption);
    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.01, 0.01, 0.01),
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..10_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    let mut delete_beams_system = atomecs::dipole::transition_switcher::DisableBlueMOTBeamsSystem;
    delete_beams_system.run_now(&world);
    println!("Switched off blue MOT");

    for _i in 0..10_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    let mut switcher_system =
        atomecs::dipole::transition_switcher::AttachNewAtomicTransitionToAtomsSystem;
    switcher_system.run_now(&world);
    println!("Switched to red MOT");

    for _i in 0..50_000 {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
