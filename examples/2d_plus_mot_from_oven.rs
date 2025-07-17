//! A 2D+ mot configuration, loaded directly from oven.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::Atom;
use lib::atom::{Position, Velocity};
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::atom_sources::oven::{OvenAperture, OvenBuilder};
use lib::atom_sources::{VelocityCap, AtomSourcePlugin};
use lib::atom_sources::surface::SurfaceSource;
use lib::destructor::ToBeDestroyed;
use lib::integrator::Timestep;
use lib::laser::LaserPlugin;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file::{FileOutputPlugin};
use lib::output::file::Text;
use lib::shapes::Cuboid;
use lib::shapes::Cylinder;
use lib::sim_region::{SimulationVolume, VolumeType};
use lib::simulation::SimulationBuilder;
use lib::species::{Strontium88, Strontium88_461};
use nalgebra::Vector3;
use bevy::prelude::*;
use std::time::Instant;
use std::marker::PhantomData;

const BEAM_NUMBER : usize = 6;


fn main() {
    let now = Instant::now();

    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugins(LaserCoolingPlugin::<Strontium88_461, {BEAM_NUMBER}>::default());
    sim_builder.add_plugins(AtomSourcePlugin::<Strontium88>::default());
    sim_builder.add_plugins(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(), 10));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(), 10));
    let mut sim = sim_builder.build();

    // Create magnetic field.
    sim.world_mut()
        .spawn(QuadrupoleField3D::gauss_per_cm(65.0, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });

    // Push beam along z
    let push_beam_radius = 1e-3;
    let push_beam_power = 0.010;
    let push_beam_detuning = 0.0;

    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: push_beam_radius,
            power: push_beam_power,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            push_beam_detuning,
            -1,
        ));

    // Create cooling lasers.
    let detuning = -45.0;
    let power = 0.23;
    let radius = 33.0e-3 / (2.0 * 2.0_f64.sqrt()); // 33mm 1/e^2 diameter
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning, 1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, -1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning, 1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, 1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning, 1,
        ));
    sim.world_mut()
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            detuning, 1,
        ));

    // Create an oven.
    // The oven will eject atoms on the first frame and then be deleted.
    let number_to_emit = 400000;
    sim.world_mut()
        .spawn(OvenBuilder::<Strontium88>::new(776.0, Vector3::x())
            .with_aperture(OvenAperture::Circular {
                radius: 0.005,
                thickness: 0.001,
            })
            .build())
        .insert(Position {
            pos: Vector3::new(-0.083, 0.0, 0.0),
        })
        .insert(MassDistribution::new(vec![MassRatio {
            mass: 88.0,
            ratio: 1.0,
        }]))
        .insert(AtomNumberToEmit {
            number: number_to_emit,
        })
        .insert(ToBeDestroyed);

    // // Surface Source example.
    // sim.world_mut()
    //     .spawn(SurfaceSource::<Strontium88>{
    //         temperature: 776.0,
    //         phantom: PhantomData,
    //     })
    //     .insert(Position {
    //         pos: Vector3::new(0.0, 0.0, 0.0),
    //     })
    //     .insert(MassDistribution::new(vec![MassRatio {
    //         mass: 88.0,
    //         ratio: 1.0,
    //     }]))
    //     .insert(AtomNumberToEmit {
    //         number: number_to_emit,
    //     })
    //     .insert(Cylinder::new(0.005, 0.001, Vector3::x()))
    //     .insert(ToBeDestroyed);

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1.0e-6 });

    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation.
    sim.world_mut()
        .spawn(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .insert(Cuboid {
            half_width: Vector3::new(0.1, 0.01, 0.01), 
        })
        .insert(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        });

    // The simulation bound also now includes a small pipe to capture the 2D MOT output properly.
    sim.world_mut()
        .spawn(Position {
            pos: Vector3::new(0.0, 0.0, 0.1),
        })
        .insert(Cuboid {
            half_width: Vector3::new(0.01, 0.01, 0.1),
        })
        .insert(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        });

    // Also use a velocity cap so that fast atoms are not even simulated.
    sim.world_mut().insert_resource(VelocityCap { value: 200.0 });

    // Run the simulation for a number of steps.
    for _i in 0..10_000 {
        sim.update();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
