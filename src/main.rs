extern crate magneto_optical_trap as lib;


use lib::detector;
use lib::simulation_templates::mot_2d_plus::create;
fn main(){
    let (mut world, mut dispatcher) = create();

    for _i in 0..10 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
    //detector::clearcsv("detector.csv");
    //detector::print_detected_to_file("detector.csv", &vec![1.0,2.0,3.0,4.0,5.0]);
}
