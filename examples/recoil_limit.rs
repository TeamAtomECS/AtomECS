//! Simulation of atoms cooled to the Doppler limit.

extern crate atomecs as lib;
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
use lib::output::file;
use lib::output::file::Text;
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
use specs::{Builder, World};
use std::fs::read_to_string;
use std::time::Instant;

extern crate serde;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RecoilSimulationConfiguration {
    /// Detuning of laser beams, in units of kHz.
    pub detuning: f64,
    /// Number of simulation steps to evolve for.
    pub number_of_steps: i32,
    /// Peak intensity of the beams, as saturation intensity
    pub i_over_isat: f64,
    /// Quadrupole gradient along z axis, in units of G/cm
    pub quad_grad: f64,
}
impl Default for RecoilSimulationConfiguration {
    fn default() -> Self {
        RecoilSimulationConfiguration {
            detuning: -50.0,
            number_of_steps: 100_000,
            i_over_isat: 1.9,
            quad_grad: 4.0,
        }
    }
}

fn recoil_limited_transition() -> AtomicTransition {
    AtomicTransition::strontium_red()
}

fn main() {
    let now = Instant::now();

    let read_result = read_to_string("recoil.json");
    let configuration: RecoilSimulationConfiguration = match read_result {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap(),
        Err(_) => RecoilSimulationConfiguration::default(),
    };

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 50),
        "",
        &[],
    );

    builder = builder.with(
        file::new::<Position, Text>("pos.txt".to_string(), 50),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(4.0, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    // Create cooling lasers.
    let detuning = configuration.detuning * 1.0e-3; //0.5 * recoil_limited_transition().linewidth / 1.0e6;
    let intensity = recoil_limited_transition().saturation_intensity * configuration.i_over_isat;
    let radius = 0.03;
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    world
        .create_entity()
        .with(GaussianBeam::from_peak_intensity(
            beam_centre.clone(),
            Vector3::new(0.0, 0.0, 1.0),
            intensity,
            radius,
        ))
        .with(CoolingLight::for_species(
            recoil_limited_transition(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam::from_peak_intensity(
            beam_centre.clone(),
            Vector3::new(0.0, 0.0, -1.0),
            intensity,
            radius,
        ))
        .with(CoolingLight::for_species(
            recoil_limited_transition(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam::from_peak_intensity(
            beam_centre.clone(),
            Vector3::new(1.0, 0.0, 0.0),
            intensity,
            radius,
        ))
        .with(CoolingLight::for_species(
            recoil_limited_transition(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam::from_peak_intensity(
            beam_centre.clone(),
            Vector3::new(-1.0, 0.0, 0.0),
            intensity,
            radius,
        ))
        .with(CoolingLight::for_species(
            recoil_limited_transition(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam::from_peak_intensity(
            beam_centre.clone(),
            Vector3::new(0.0, 1.0, 0.0),
            intensity,
            radius,
        ))
        .with(CoolingLight::for_species(
            recoil_limited_transition(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam::from_peak_intensity(
            beam_centre.clone(),
            Vector3::new(0.0, -1.0, 0.0),
            intensity,
            radius,
        ))
        .with(CoolingLight::for_species(
            recoil_limited_transition(),
            detuning,
            1,
        ))
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 2e-6 });

    // Add atoms
    for _ in 0..3000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Force::new())
            .with(Mass { value: 88.0 })
            .with(recoil_limited_transition())
            .with(Atom)
            .with(NewlyCreated)
            .build();
    }

    // Enable fluctuation options
    //  * Allow photon numbers to fluctuate.
    //  * Allow random force from emission of photons.
    world.add_resource(EmissionForceOption::default());
    world.add_resource(ScatteringFluctuationsOption::On);
    world.add_resource(lib::gravity::ApplyGravityOption);

    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(1e-3, 1e-3, 1e-3),
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // Run the simulation for a number of steps.
    for _i in 0..configuration.number_of_steps {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
