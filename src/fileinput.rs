use std::fs::File;
use std::io::prelude::*;
use yaml_rust::yaml::{Hash, Yaml};
use yaml_rust::YamlLoader;


fn load_file(file: &str) {
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let docs = YamlLoader::load_from_str(&contents).unwrap();

    // TO DO
	// will complete after the ecs.rs is checked
}
pub struct Parameters2D{
	pub laser1:Laserparameter,
	pub laser2:Laserparameter,
	pub laser3:Laserparameter,
	pub laser4:Laserparameter,
	pub pushinglaser:Laserparameter,
	pub oven:Ovenparameter,
	pub magnetic:Magneticparameter,
}

pub enum Magneticparameter{
	Quodrapoleparameter{gradient:f64,centre:[f64;3]},
}

pub struct Ovenparameter{
	temperature:f64,
	direction:[f64;3],
}

pub struct Laserparameter{
	frequency:f64,
	direction:[f64;3],
	intensity:f64,
	e_radius:f64,
}
