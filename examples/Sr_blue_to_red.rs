//! Loading a Sr cross beam dipole trap from center.

extern crate atomecs as lib;
extern crate nalgebra;
use crate::lib::laser_cooling::force::EmissionForceOption;
use atomecs::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
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
    let delays = [1000, 3000, 7500, 12500, 20000, 50000];

    for i in 0..delays.len() {
        let now = Instant::now();
        println!("Doing the sim for a delay of {} µs.", delays[i]);

        // Create the simulation world and builder for the ECS dispatcher.
        let mut world = World::new();
        ecs::register_components(&mut world);
        ecs::register_resources(&mut world);
        let mut builder = ecs::create_simulation_dispatcher_builder();

        let posname = format!("pos_lb5-{}.txt", delays[i]);
        let velname = format!("vel_lb5-{}.txt", delays[i]);
        let xyzname = format!("pos_lb5-{}.xyz", delays[i]);
        // Configure simulation output.
        builder = builder.with(file::new::<Position, Text>(posname, 100), "", &[]);
        builder = builder.with(file::new::<Velocity, Text>(velname, 100), "", &[]);
        builder = builder.with(
            file::new_with_filter::<Position, XYZ, Atom>(xyzname, 100),
            "",
            &[],
        );

        let mut dispatcher = builder.build();
        dispatcher.setup(&mut world);
        // BEGIN MOT PART

        world.register::<Ramp<QuadrupoleField3D>>();

        let mut magnetic_frames = Vec::new();
        magnetic_frames.push((0.0, QuadrupoleField3D::gauss_per_cm(60.0, Vector3::x())));
        magnetic_frames.push((0.01, QuadrupoleField3D::gauss_per_cm(60.0, Vector3::x())));
        magnetic_frames.push((
            0.01 + delays[i] as f64 * 1.0e-6,
            QuadrupoleField3D::gauss_per_cm(1.0, Vector3::x()),
        ));
        let magnetic_ramp = Ramp::new(magnetic_frames);

        world
            .create_entity()
            .with(QuadrupoleField3D::gauss_per_cm(60.0, Vector3::x()))
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0e-6),
            })
            .with(magnetic_ramp)
            .build();

        // BLUE MOT
        lib::helper_files::blue_beams::add_blue_mot_beams(&mut world);

        // RED MOT,
        lib::helper_files::red_beams::add_red_mot_beams(&mut world);
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
                half_width: Vector3::new(0.005, 0.005, 0.005),
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

        for _i in 0..delays[i] / 2 as i64 {
            dispatcher.dispatch(&mut world);
            world.maintain();
        }
        let mut delete_beams_system =
            atomecs::dipole::transition_switcher::DisableBlueMOTBeamsSystem;
        delete_beams_system.run_now(&world);
        println!("Switched off blue MOT");

        for _i in 0..delays[i] / 2 as i64 {
            dispatcher.dispatch(&mut world);
            world.maintain();
        }

        let mut switcher_system =
            atomecs::dipole::transition_switcher::AttachNewAtomicTransitionToAtomsSystem;
        switcher_system.run_now(&world);

        println!("Switched to red MOT");

        for _i in 0..80_000 {
            dispatcher.dispatch(&mut world);
            world.maintain();
        }

        println!("Simulation completed in {} ms.", now.elapsed().as_millis());
    }
}
