extern crate atomecs as lib;
extern crate nalgebra;
use nalgebra::Vector3;
use bevy::prelude::*;
use std::time::Instant;
use std::marker::PhantomData;
use lib::constant::PI;
use lib::atom::{Atom, Position, Velocity};
use lib::integrator::Timestep;
use lib::output::file::FileOutputPlugin;
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use lib::sim_region::{SimulationVolume, VolumeType};
use lib::shapes::{Cylinder as MyCylinder, CylindricalPipe};
use lib::species::Strontium88;
use lib::atom_sources::{AtomSourcePlugin, WeightedProbabilityDistribution, VelocityCap};
use lib::atom_sources::emit::{EmitFixedRate, AtomNumberToEmit};
use lib::atom_sources::mass::{MassRatio, MassDistribution};
use lib::atom_sources::oven::{OvenAperture, Oven};
use lib::collisions::{CollisionPlugin, ApplyAtomCollisions, ApplyWallCollisions};
use lib::collisions::wall_collisions::{WallData, WallType};

fn main() {
    let now = Instant::now();
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(AtomSourcePlugin::<Strontium88>::default());
    sim_builder.add_plugins(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(),10));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(),10));
    // sim_builder.add_plugins(FileOutputPlugin::<NumberOfWallCollisions, Text, Atom>::new("wall_collisions.txt".to_string(),1));
    sim_builder.add_plugins(CollisionPlugin);

    let mut sim = sim_builder.build();

    sim.world_mut().insert_resource(ApplyAtomCollisions(false));
    sim.world_mut().insert_resource(ApplyWallCollisions(true));

    sim.world_mut()
        .spawn(WallData{
            wall_type: WallType::Rough})
        .insert(CylindricalPipe::new(1e-5, 400e-6, Vector3::new(1.0, 0.0, 0.0)))
        .insert(Position{pos: Vector3::new(-200e-6, 0.0, 0.0)});

    sim.world_mut()
        .spawn(SimulationVolume{volume_type: VolumeType::Inclusive})
        .insert(MyCylinder::new(110e-5, 2000e-6, Vector3::new(1.0, 0.0, 0.0)))
        .insert(Position{pos: Vector3::new(600e-6, 0.0, 0.0)});

    let number_to_emit = 1e10;

    let mut thetas = Vec::<f64>::new();
    let mut weights = Vec::<f64>::new();

    let n = 10000;
    for i in 0..n {
        let theta = (i as f64) / (n as f64 + 1.0) * PI / 2.0;
        let weight = theta.sin() * theta.cos();
        thetas.push(theta);
        weights.push(weight);
    }

    let uniform_distribution = WeightedProbabilityDistribution::new(thetas, weights);

    sim.world_mut()
        .spawn(Oven::<Strontium88> {
            temperature: 700.0,
            aperture: OvenAperture::Circular{radius: 1e-5, thickness: 1e-9},
            direction: Vector3::new(1.0, 0.0, 0.0),
            theta_distribution: uniform_distribution,
            max_theta: PI/2.0,
            phantom: PhantomData,
        })
        .insert(Position {
            pos: Vector3::new(-39999e-8, 0.0, 0.0),
        })
        .insert(MassDistribution::new(vec![MassRatio {
            mass: 88.0,
            ratio: 1.0,
        }]))
        .insert(AtomNumberToEmit{number: 0})
        .insert(EmitFixedRate{rate: number_to_emit});

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 1e-7 });
    sim.world_mut().insert_resource(VelocityCap {value: f64::MAX});

    // Run the simulation for a number of steps.
    for _i in 0..1_000 {
        sim.update();
    }
    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
