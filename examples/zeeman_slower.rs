extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use nalgebra::Vector3;
use specs::{Builder, World};

fn main() {
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut dispatcher = ecs::create_simulation_dispatcher();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(100.0))
        .with(Position::new())
        .build();

    // Create a cooling laser, pointing downwards.
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 10.0,
            direction: -Vector3::z(),
        })
        .with(CoolingLight::for_species(AtomInfo::rubidium(), -6.0, 1.0))
        .build();

    // Create an atom
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, -0.01),
        })
        .with(Atom)
        .with(Force::new())
        .with(Velocity {
            vel: Vector3::new(0.0, 0.0, 50.0),
        })
        .with(NewlyCreated)
        .with(AtomInfo::rubidium())
        .with(Mass { value: 87.0 })
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    // Run the simulation for a number of steps.
    for _i in 0..50000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
}
