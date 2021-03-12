use criterion::{criterion_group, criterion_main, Criterion};
extern crate atomecs as lib;
extern crate specs;

extern crate nalgebra;
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::force::EmissionForceOption;
use lib::laser::gaussian::GaussianBeam;
use lib::laser::photons_scattered::ScatteringFluctuationsOption;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use nalgebra::Vector3;
use rand::distributions::{Distribution, Normal};
use specs::{Builder, DispatcherBuilder, World};

fn criterion_benchmark(c: &mut Criterion) {
    // Mock up a simulation world and dispatcher
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut dispatcher = ecs::create_simulation_dispatcher_builder().build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(18.2, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    // Create cooling lasers.
    let detuning = -3.0;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 0.0, 1.0),
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
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 0.0, -1.0),
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
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(1.0, 0.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, 1.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: Vector3::new(0.0, -1.0, 0.0),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            detuning,
            1,
        ))
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    let vel_dist = Normal::new(0.0, 0.22);
    let pos_dist = Normal::new(0.0, 1.2e-4);
    let mut rng = rand::thread_rng();

    // Add atoms
    for _ in 0..10000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng),
                ),
            })
            .with(Velocity {
                vel: Vector3::new(
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng),
                ),
            })
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(AtomicTransition::rubidium())
            .with(Atom)
            .with(NewlyCreated)
            .build();
    }

    // Enable fluctuation options
    //  * Allow photon numbers to fluctuate.
    //  * Allow random force from emission of photons.
    world.add_resource(EmissionForceOption::default());
    world.add_resource(ScatteringFluctuationsOption::default());

    // Run a few times with the standard dispatcher to create atoms, initialise components, etc.
    for _ in 0..5 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    // Now bench just a specific system.
    let mut bench_builder = DispatcherBuilder::new();
    bench_builder.add(lib::laser::rate::CalculateRateCoefficientsSystem, "", &[]);
    // Configure thread pool.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build()
        .unwrap();

    bench_builder.add_pool(::std::sync::Arc::new(pool));
    let mut bench_dispatcher = bench_builder.build();

    c.bench_function("rate_calculation", |b| {
        b.iter(|| bench_dispatcher.dispatch(&mut world.res))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
