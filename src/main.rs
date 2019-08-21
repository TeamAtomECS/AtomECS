extern crate magneto_optical_trap as lib;

use lib::detector;
extern crate specs;

use lib::fileinput::write_file_template;
use lib::simulation_templates::loadfromconfig::create_from_config;
use specs::Builder;

use lib::optimization::OptEarly;

use lib::simulation_templates::mot_2d_plus::create;

use specs::RunNow;
use std::time::{Duration, Instant};

//use std::io::stdin;
fn main() {
    //let mut s=String::new();
    //stdin()
    //    .read_line(&mut s)
    //    .expect("Did not enter a correct string");
    let now = Instant::now();

    let (mut world, mut dispatcher) = create_from_config("example.yaml");
    world
        .create_entity()
        .with(OptEarly {
            timethreshold: 2e-4,
            if_opt: false,
        })
        .build();
    //let (mut world, mut dispatcher) = create();
    for _i in 0..50000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
    let mut output = detector::PrintOptResultSystem;
    output.run_now(&world.res);
    println!("time taken to run{}", now.elapsed().as_millis());
    //write_file_template("example.yml")
    //detector::clearcsv("detector.csv");
    //detector::print_detected_to_file("detector.csv", &vec![1.0,2.0,3.0,4.0,5.0]);
}
