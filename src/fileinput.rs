use std::fs::File;
use std::io::prelude::*;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
use crate::atom::AtomInfo;
use crate::atom_sources::mass::{MassDistribution, MassRatio};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::io::BufWriter;

pub fn load_file(file: &str) -> SimArchetype {
	let file = File::open(file).expect("Unable to open file");
	let deserialized: SimArchetype = serde_yaml::from_reader(file).expect("Could not read");
	deserialized
	//println!("{}", deserialized.lasers.get(0).expect("empty array").beam.e_radius);
}

/// Writes a YAML file for 2D plus MOT.
/// use this as the the input format in detail
pub fn write_file_template(file: &str) {
	let file = File::create(file).expect("Unable to open file");
	let mut writer = BufWriter::new(file);

	let lasers = vec![
		LaserArchetype {
			direction: Vector3::new(1., 0., 0.),
			frequency: 1e10,
			polarization: 1.,
			power: 10.,
			e_radius: 0.1,
			intersection: Vector3::new(0., 0., 1.),
		},
		LaserArchetype {
			direction: Vector3::new(-1., 0., 0.),
			frequency: 1e10,
			polarization: -1.,
			power: 10.,
			e_radius: 0.1,
			intersection: Vector3::new(0., 2., 1.),
		},
	];
	let ovens = vec![OvenArchetype {
		position: Vector3::new(1., 0., 0.),
		rate: 100.,
		direction: Vector3::new(0., 0., 1.0),
		temperature: 300.,
		radius_aperture: 0.01,
		thickness: 0.01,
	}];
	let mag = MagArchetype {
		centre: Vector3::new(1., 0., 0.),
		gradient: 0.011,
		uniform: Vector3::new(0., 0., 2.),
	};
	let massrubidium = MassDistribution::new(vec![
		MassRatio {
			mass: 87.,
			ratio: 0.2783,
		},
		MassRatio {
			mass: 85.,
			ratio: 0.7217,
		},
	]);
	let sim = SimArchetype {
		lasers: lasers,
		ovens,
		magnetic: mag,
		mass: massrubidium,
		atominfo: AtomInfo::rubidium(),
	};

	let serialized = serde_yaml::to_string(&sim).unwrap();
	write!(writer, "{}", serialized.to_string()).expect("Could not write to file.");
}

/// A laser beam
#[derive(Deserialize, Serialize)]
pub struct LaserArchetype {
	pub direction: Vector3<f64>,
	pub frequency: f64,
	pub polarization: f64,
	pub power: f64,
	pub e_radius: f64,
	pub intersection: Vector3<f64>,
}

/// An oven
#[derive(Deserialize, Serialize)]
pub struct OvenArchetype {
	pub position: Vector3<f64>,
	pub rate: f64,
	pub direction: Vector3<f64>,
	pub temperature: f64,
	pub radius_aperture: f64,
	pub thickness: f64,
}

/// Magnetic fields used
#[derive(Deserialize, Serialize)]
pub struct MagArchetype {
	pub centre: Vector3<f64>,
	pub gradient: f64,
	pub uniform: Vector3<f64>,
}

#[derive(Deserialize, Serialize)]
pub struct SimArchetype {
	pub lasers: Vec<LaserArchetype>,
	pub ovens: Vec<OvenArchetype>,
	pub atominfo: AtomInfo,
	pub mass: MassDistribution,
	pub magnetic: MagArchetype,
}
