//! Simulate a 1D MOT.
//!
//! The 1D MOT is formed by counter-propagating laser beams along the z-axis.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Binary;
use nalgebra::Vector3;
use specs::{Builder, World};

fn main() {
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Position, Binary>("pos.dat".to_string(), 10),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Binary>("vel.dat".to_string(), 10),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(15.0, Vector3::z()))
        .with(Position::new())
        .build();

    // Create cooling lasers.
    let detuning = -12.0;
    let power = 0.03;
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: power,
            direction: -Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: power,
            direction: Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            -1,
        ))
        .build();

    // Create atoms
    for i in 0..20 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, -0.05),
            })
            .with(Atom)
            .with(Force::new())
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 10.0 + (i as f64) * 5.0),
            })
            .with(NewlyCreated)
            .with(AtomicTransition::rubidium())
            .with(Mass { value: 87.0 })
            .build();
    }

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    // Run the simulation for a number of steps.
    for _i in 0..5000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
}
