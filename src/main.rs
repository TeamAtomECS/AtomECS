extern crate magneto_optical_trap as lib;
extern crate nalgebra;

use lib::detector;
extern crate specs;
#[allow(unused_imports)]
use lib::simulation_templates::loadfromconfig::create_from_config;

use lib::laser::force::RandomWalkMarker;
use lib::optimization::LargerEarlyTimestepOptimization;
use lib::atom::Position;
use lib::laser::repump::RepumpLoss;

use lib::atom_sources::oven::OvenVelocityCap;
use lib::sim_region::{Cuboid, VolumeType};

#[allow(unused_imports)]

use nalgebra::Vector3;
use specs::{RunNow, Builder};
#[allow(unused_imports)]
use std::time::{Duration, Instant};

//use std::io::stdin;
fn main() {
    //let mut s=String::new();
    //stdin()
    //    .read_line(&mut s)
    //    .expect("Did not enter a correct string");
    let now = Instant::now();
    let (mut world, mut dispatcher) = create_from_config("example.yaml");

    //increase the timestep at the begining of the simulation
    world.add_resource(LargerEarlyTimestepOptimization::new(2e-4));
    //include random walk(Optional)
    world.add_resource(RandomWalkMarker { value: true });
    
    world.create_entity().with(Position { pos: Vector3::new(0.0,0.0,0.0)}).with(Cuboid { half_width: Vector3::new(0.1,0.1,0.1), vol_type: VolumeType::Inclusive}).build();

    world.add_resource(OvenVelocityCap { cap: 1000. });
    world.add_resource(RepumpLoss { proportion: 0.0 });
    //let (mut world, mut dispatcher) = create();
    for _i in 0..50000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
    let mut output = detector::PrintDetectResultSystem;
    output.run_now(&world.res);
    println!("time taken to run in ms{}", now.elapsed().as_millis());
    //write_file_template("example.yml")
}
