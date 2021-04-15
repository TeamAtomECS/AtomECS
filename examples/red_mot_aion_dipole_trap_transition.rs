//! Loading a Sr cross beam dipole trap from center.

extern crate atomecs as lib;
extern crate nalgebra;
use crate::laser::cooling::CoolingLight;
use crate::lib::laser::force::EmissionForceOption;
use atomecs::laser::photons_scattered::ScatteringFluctuationsOption;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::central_creator::CentralCreator;
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::destructor::ToBeDestroyed;
use lib::dipole;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file::Text;
use lib::output::{file, xyz_file};
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
use specs::{Builder, RunNow, World};
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
        file::new::<Position, Text>("pos_dipole_aion_fine.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel_dipole_aion_fine.txt".to_string(), 100),
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
            discard_place: Vector3::new(20., 20., 20.),
            name: format!("{}", "aion_transition_gravity_1000_test_new_git_fine"),
        })
        .build();
    // BEGIN MOT PART

    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(1.0, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0e-6),
        })
        .build();

    let detuning = -0.12; //MHz
    let power = 0.01; //W total power of all Lasers together
    let radius = 1.0e-2 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    // Horizontal beams along z
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::z(),
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
            direction: -Vector3::z(),
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
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
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
            direction: Vector3::new(1.0, -1.0, 0.0).normalize(),
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
            direction: Vector3::new(-1.0, 1.0, 0.0).normalize(),
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
            direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world.add_resource(EmissionForceOption::default());
    world.add_resource(ScatteringFluctuationsOption::default());

    // END MOT part

    // Create dipole laser.
    let power = 10.;
    let e_radius = 100.0e-6 / (2.0_f64.sqrt());

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
        direction: Vector3::new(0.924, 0.259, 1.).normalize(),
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::gaussian::GaussianReferenceFrame {
            x_vector: Vector3::new(0., 0.96810035, -0.25056281),
            y_vector: Vector3::new(-0.74536307, 0.16703989, 0.64539258),
            ellipticity: 0.0,
        })
        .with(laser::gaussian::make_gaussian_rayleigh_range(
            &1064.0e-9,
            &gaussian_beam,
        ))
        .build();
    // creating the entity that represents the source
    //
    // contains a central creator
    let number_to_emit = 1_000;
    let size_of_cube = 1.0e-4;
    let speed = 0.01; // m/s

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
    world.add_resource(Timestep { delta: 1.0e-7 });

    //enable gravity
    world.add_resource(lib::gravity::ApplyGravityOption);
    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.001, 0.001, 0.001),
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..200_000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
    let mut ramp_down_system = dipole::transition_switcher::RampMOTBeamsSystem;
    world.add_resource(dipole::transition_switcher::MOTAbsoluteDetuningRampRate {
        absolute_rate: 2.0e6, // Hz / s
    });
    world.add_resource(dipole::transition_switcher::MOTRelativePowerRampRate {
        relative_rate: 1.0e-60, // 1 / s
    });

    let mut switcher_system =
        dipole::transition_switcher::AttachAtomicDipoleTransitionToAtomsSystem;
    switcher_system.run_now(&world.res);
    for _i in 0..300_000 {
        dispatcher.dispatch(&mut world.res);
        ramp_down_system.run_now(&world.res);
        world.maintain();
    }
    let mut delete_beams_system = dipole::transition_switcher::DisableMOTBeamsSystem;
    delete_beams_system.run_now(&world.res);
    println!("Switched from MOT to Dipole setup");
    for _i in 0..400_000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
