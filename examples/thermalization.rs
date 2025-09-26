extern crate atomecs as lib;
extern crate nalgebra;
use nalgebra::Vector3;
use rand_distr::{Distribution, Uniform};
use bevy::prelude::*;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::output::file::FileOutputPlugin;
use lib::output::file::Text;
use lib::simulation::SimulationBuilder;
use std::fs::File;
use std::io::{Error, Write};
use std::time::Instant;
use lib::shapes::Sphere as MySphere;
use lib::collisions::wall_collisions::{WallData, WallType};
use lib::collisions::CollisionPlugin;
use lib::collisions::atom_collisions::{CollisionParameters, CollisionsTracker};
use lib::sim_region::{SimulationVolume, VolumeType};

fn main() {
    let now = Instant::now();
    let mut sim_builder = SimulationBuilder::default();
    sim_builder.add_plugins(FileOutputPlugin::<Position, Text, Atom>::new("pos.txt".to_string(),150));
    sim_builder.add_plugins(FileOutputPlugin::<Velocity, Text, Atom>::new("vel.txt".to_string(),150));
    sim_builder.add_plugins(CollisionPlugin);

    let mut sim = sim_builder.build();

    let p_dist = Uniform::new(-25e-3, 25e-3).unwrap();
    let v_dist = Uniform::new(-5e-1, 5e-1).unwrap();
    for _i in 0..25000 {
        sim.world_mut()
            .spawn(Position {
                pos: Vector3::new(
                    p_dist.sample(&mut rand::rng()),
                    p_dist.sample(&mut rand::rng()),
                    p_dist.sample(&mut rand::rng()),
                ),
            })
            .insert(Atom)
            .insert(Force::default())
            .insert(Velocity {
                vel: Vector3::new(
                    v_dist.sample(&mut rand::rng()),
                    v_dist.sample(&mut rand::rng()),
                    v_dist.sample(&mut rand::rng()),
                ).normalize(),
            })
            .insert(NewlyCreated)
            .insert(Mass { value: 87.0 });
    }

    sim.world_mut().insert_resource(CollisionParameters {
        macroparticle: 5e8,
        box_number: 100, //Any number large enough to cover entire cloud with collision boxes. Overestimating box number will not affect performance.
        box_width: 1e-2, //Too few particles per box will both underestimate collision rate and cause large statistical fluctuations.
        //Boxes must also be smaller than typical length scale of density variations within the cloud, since the collisions model treats gas within a box as homogeneous.
        sigma: 3.5e-16, //Approximate collisional cross section of Rb87
        collision_limit: 10_000_000.0, //Maximum number of collisions that can be calculated in one frame.
                                       //This avoids absurdly high collision numbers if many atoms are initialised with the same position, for example.
    });

    sim.world_mut().insert_resource(CollisionsTracker {
        num_collisions: Vec::new(),
        num_atoms: Vec::new(),
        num_particles: Vec::new(),
    });

    sim.world_mut()
        .spawn(WallData{
            wall_type: WallType::Rough})
        .insert(MySphere{radius: 5e-2})
        .insert(Position{pos: Vector3::new(0.0, 0.0, 0.0)});

    sim.world_mut()
        .spawn(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .insert(MySphere {
            radius: 502e-4, 
        })
        .insert(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        });

    // Define timestep
    sim.world_mut().insert_resource(Timestep { delta: 5e-5 });

    let mut filename = File::create("collisions.txt").expect("Cannot create file.");

    // Run the simulation for a number of steps.
    for _i in 0..25_000 {
        sim.update();

        if (_i > 0) && (_i % 50_i32 == 0) {
            let tracker = sim.world().get_resource::<CollisionsTracker>().unwrap();
            let _result = write_collisions_tracker(
                &mut filename,
                &_i,
                &tracker.num_collisions,
                &tracker.num_atoms,
                &tracker.num_particles,
            )
            .expect("Could not write collision stats file.");
        }
    }
    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}

// Write collision stats to file

fn write_collisions_tracker(
    filename: &mut File,
    step: &i32,
    num_collisions: &Vec<i32>,
    num_atoms: &Vec<f64>,
    num_particles: &Vec<i32>,
) -> Result<(), Error> {
    let str_collisions: Vec<String> = num_collisions.iter().map(|n| n.to_string()).collect();
    let str_atoms: Vec<String> = num_atoms.iter().map(|n| format!("{:.2}", n)).collect();
    let str_particles: Vec<String> = num_particles.iter().map(|n| n.to_string()).collect();
    write!(
        filename,
        "{:?}\r\n{:}\r\n{:}\r\n{:}\r\n",
        step,
        str_collisions.join(" "),
        str_atoms.join(" "),
        str_particles.join(" ")
    )?;
    Ok(())
}

