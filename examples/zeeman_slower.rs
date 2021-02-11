extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use specs::{Builder, World};

fn main() {
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Add some output to the simulation
    builder = builder.with(
        file::new::<Position, Text>("pos.txt".to_string(), 10),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 10),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(100.0, Vector3::z()))
        .with(Position::new())
        .build();

    // Create a cooling laser, pointing downwards.
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 0.5,
            direction: -Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            -6.0,
            -1.0,
        ))
        .build();

    // Create atoms
    for i in 0..20 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, -0.03),
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