extern crate magneto_optical_trap as lib;

use lib::detector;

use lib::fileinput::write_file_template;
use lib::simulation_templates::loadfromconfig::create_from_config;
fn main() {
    let (mut world, mut dispatcher) =create_from_config();

    for _i in 0..10 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
    //write_file_template("example.yml")
    //detector::clearcsv("detector.csv");
    //detector::print_detected_to_file("detector.csv", &vec![1.0,2.0,3.0,4.0,5.0]);
}
