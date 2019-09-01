

use crate::detector;
extern crate specs;

use crate::fileinput::write_file_template;
use crate::simulation_templates::loadfromconfig::create_from_config;
use specs::Builder;

use crate::laser::force::RandomWalkMarker;
use crate::optimization::OptEarly;

use crate::laser::repump::RepumpLoss;

use crate::atom_sources::oven::VelocityCap;
use crate::destructor::BoundaryMarker;
use crate::simulation_templates::mot_2d_plus::create;
use specs::RunNow;
use std::time::{Duration, Instant};

use crate::output::file_output::FileOutputMarker;

//use std::io::stdin;
fn main() {
    //let mut s=String::new();
    //stdin()
    //    .read_line(&mut s)
    //    .expect("Did not enter a correct string");
    let now = Instant::now();
    let (mut world, mut dispatcher) = create_from_config("example.yaml");

    //increase the timestep at the begining of the simulation
    world.add_resource(OptEarly::new(2e-4));
    //include random walk(Optional)
    world.add_resource(RandomWalkMarker { value: true });

    //include boundary (walls)

    world.add_resource(BoundaryMarker { value: true });
    world.add_resource(VelocityCap { cap: 1000. });
    world.add_resource(RepumpLoss { proportion: 0.0 });
    world.add_resource(FileOutputMarker { value: false });
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
