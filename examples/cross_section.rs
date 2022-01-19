//!

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::DEFAULT_BEAM_LIMIT;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::photons_scattered::ExpectedPhotonsScatteredVector;
use lib::laser_cooling::CoolingLight;
use lib::laser_cooling::transition::AtomicTransition;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use lib::species::{Rubidium87_780D2, Rubidium87};
use nalgebra::Vector3;
use specs::prelude::*;

fn main() {

    let mut sim_builder = SimulationBuilder::default::<Rubidium87_780D2, Rubidium87>();
    sim_builder.add_plugin(FileOutputPlugin::<ExpectedPhotonsScatteredVector<Rubidium87_780D2, {DEFAULT_BEAM_LIMIT}>, Text, Atom>::new("scattered.txt".to_string(), 10));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Set the intensity equal to Isat.
    let radius = 0.01; // 1cm
    let std = radius / 2.0_f64.powf(0.5);
    let intensity = Rubidium87_780D2::saturation_intensity();
    let power = 2.0 * lib::constant::PI * std.powi(2) * intensity;

    // Single laser beam propagating in +x direction.
    let detuning = 0.0;
    //let power = 3e-3; //3mW
    sim.world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::x(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        ))
        .build();

    // Define timestep
    sim.world.insert(Timestep { delta: 1.0e-6 });

    // Create atoms
    for i in 0..200 {
        sim.world
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
            .with(Rubidium87_780D2)
            .with(Mass { value: 87.0 })
            .build();
    }

    // Run the simulation
    for _i in 0..4 {
        sim.step();
    }
}
