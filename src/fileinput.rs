use std::fs::File;
use std::io::prelude::*;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
use crate::atom::{AtomInfo, Mass, Position};
use crate::laser::cooling::CoolingLight;
use crate::laser::gaussian::GaussianBeam;
use crate::oven::Oven;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Read};

pub fn load_file(file: &str) {
	let mut file = File::open(file).expect("Unable to open file");
	let deserialized : SimArchetype = serde_yaml::from_reader(file).expect("Could not read");
	println!("{}", deserialized.lasers.get(0).expect("empty array").beam.e_radius);
}

/// Writes a YAML file. This is so you can check the syntax to use.
pub fn write_file(file: &str) {
	let mut file = File::create(file).expect("Unable to open file");
	let mut writer = BufWriter::new(file);

	let lasers = vec![
		LaserArchetype {
			light: CoolingLight::for_species(AtomInfo::rubidium(), -6.0, 1.0),
			beam: GaussianBeam {
				intersection: Vector3::new(0.0, 0.0, 0.0),
				e_radius: 0.01,
				power: 1.0,
				direction: -Vector3::z(),
			},
		},
		LaserArchetype {
			light: CoolingLight::for_species(AtomInfo::rubidium(), -6.0, 1.0),
			beam: GaussianBeam {
				intersection: Vector3::new(0.0, 0.0, 0.0),
				e_radius: 0.01,
				power: 1.0,
				direction: Vector3::z(),
			},
		},
	];

	let sim = SimArchetype {
		lasers: lasers
	};

	let serialized = serde_yaml::to_string(&sim).unwrap();
	write!(writer, "{}", serialized.to_string());
}

/// A laser beam
#[derive(Deserialize, Serialize)]
struct LaserArchetype {
	pub light: CoolingLight,
	pub beam: GaussianBeam,
}

/// An atomic oven
#[derive(Deserialize, Serialize)]
struct OvenParameter{
	pub position:Vector3<f64>,
	pub rate:f64,
	pub direction:Vector3<f64>,
	pub temperature:f64,
	pub radius_aperture:f64,
}



#[derive(Deserialize, Serialize)]
struct SimArchetype {
	pub lasers: Vec<LaserArchetype>,
	//pub ovens: Vec<OvenArchetype>,
}
