extern crate magneto_optical_trap as lib;

use lib::detector;
extern crate specs;

use lib::fileinput::write_file_template;
use lib::simulation_templates::loadfromconfig::create_from_config;
use lib::simulation_templates::mot_2d_plus::create;
use specs::RunNow;
fn main() {
    let (mut world, mut dispatcher) = create_from_config();
    //let (mut world, mut dispatcher) = create();
    for _i in 0..20000 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
    let mut output = detector::PrintOptResultSystem;
    output.run_now(&world.res);
    //write_file_template("example.yml")
    //detector::clearcsv("detector.csv");
    //detector::print_detected_to_file("detector.csv", &vec![1.0,2.0,3.0,4.0,5.0]);
}
