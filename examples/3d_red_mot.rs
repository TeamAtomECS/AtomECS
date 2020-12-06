extern crate magneto_optical_trap as lib;
use lib::atom_sources::central_creator::CentralCreator;

extern crate nalgebra;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::atom_sources::oven::{OvenAperture, OvenBuilder};
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

    // creating the entity that represents the source
    //
    // contains a central creator
    let number_to_emit = 1000000;
    let size_of_cube = 1e-4;
    let speed = 1.0; // m/s

    world
        .create_entity()
        .with(CentralCreator::new_uniform_cubic(size_of_cube, speed))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 88.0,
            ratio: 1.0,
        }]))
        .with(AtomicTransition::strontium_red())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();
}
