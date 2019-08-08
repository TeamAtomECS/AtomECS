extern crate magneto_optical_trap as lib;

use lib::fileinput::{load_file, write_file};
use lib::simulation_templates::mot_2d_plus::create;

fn main() {
    let (mut world, mut dispatcher) = create();

     for _i in 0..10000 {
         dispatcher.dispatch(&mut world.res);
         world.maintain();
    }
    //write_file("example.yaml");
    //load_file("example.yaml");
}
