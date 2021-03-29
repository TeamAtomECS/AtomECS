//!

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::laser::photons_scattered::ExpectedPhotonsScatteredVector;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use specs::{Builder, World};

fn main() {
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Output scattering rate and velocity
    builder = builder.with(
        file::new::<ExpectedPhotonsScatteredVector, Text>("scattered.txt".to_string(), 1),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 1),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Set the intensity equal to Isat.
    let radius = 0.01; // 1cm
    let std = radius / 2.0_f64.powf(0.5);
    let intensity = AtomicTransition::rubidium().saturation_intensity;
    let power = 2.0 * lib::constant::PI * std.powi(2) * intensity;

    // Single laser beam propagating in +x direction.
    let detuning = 0.0;
    //let power = 3e-3; //3mW
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power,
            direction: Vector3::x(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();

    // world
    //     .create_entity()
    //     .with(lib::magnetic::uniform::UniformMagneticField::gauss(
    //         Vector3::new(20.0, 0.0, 0.0),
    //     ))
    //     .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    // Create atoms
    for i in 0..200 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Atom)
            .with(Force::new())
            .with(Velocity {
                vel: Vector3::new(-100.0 + (i as f64) * 1.0, 0.0, 0.0),
            })
            .with(NewlyCreated)
            .with(AtomicTransition::rubidium())
            .with(Mass { value: 87.0 })
            .build();
    }

    // Run the simulation twice
    dispatcher.dispatch(&mut world.res);
    world.maintain();
    dispatcher.dispatch(&mut world.res);
    world.maintain();
    dispatcher.dispatch(&mut world.res);
    world.maintain();
    dispatcher.dispatch(&mut world.res);
}
