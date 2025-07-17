//!

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::{LaserPlugin};
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::photons_scattered::ExpectedPhotonsScatteredVector;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::laser_cooling::transition::AtomicTransition;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use lib::species::{Rubidium87_780D2};
use nalgebra::Vector3;
use bevy::prelude::*;

const BEAM_NUMBER : usize = 1;

fn main() {

    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(LaserCoolingPlugin::<Rubidium87_780D2, {BEAM_NUMBER}>::default());
    sim_builder.add_plugins(FileOutputPlugin::<ExpectedPhotonsScatteredVector<Rubidium87_780D2, {BEAM_NUMBER}>, Text, Atom>::new("scattered.txt".to_string(), 2));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Set the intensity equal to Isat.
    let radius = 0.01; // 1cm
    let std = radius / 2.0_f64.powf(0.5);
    let intensity = Rubidium87_780D2::saturation_intensity();
    let power = 2.0 * lib::constant::PI * std.powi(2) * intensity;

    // Single laser beam propagating in +x direction.
    let detuning = 0.0;
    //let power = 3e-3; //3mW
    sim.world_mut()
        .spawn((GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::x(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        },CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning,
            1,
        )));

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-6 });

    // Create atoms
    for i in 0..200 {
        sim.world_mut()
            .spawn((Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            },
            Atom,
            Force::default(),
            Velocity {
                vel: Vector3::new(-100.0 + (i as f64) * 1.0, 0.0, 0.0),
            },
            NewlyCreated,
            Rubidium87_780D2,
            Mass { value: 87.0 },
            ));
    }

    // Run the simulation
    for _i in 0..3 {
        sim.update();
    }
}
