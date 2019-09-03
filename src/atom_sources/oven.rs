use crate::maths;
extern crate nalgebra;
extern crate rand;
use super::emit::AtomNumberToEmit;
use super::mass::MassDistribution;
use crate::constant;
use crate::constant::PI;
use crate::initiate::*;
use rand::Rng;

extern crate specs;
use crate::atom::*;
use nalgebra::Vector3;

use specs::{Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System};

pub fn velocity_generate(v_mag: f64, new_dir: &Vector3<f64>) -> Vector3<f64> {
	let dir = &new_dir.normalize();
	let dir_1 = new_dir.cross(&Vector3::new(2.0, 1.0, 0.5)).normalize();
	let dir_2 = new_dir.cross(&dir_1).normalize();
	let mut rng = rand::thread_rng();
	let theta = maths::jtheta_gen();
	let theta2 = rng.gen_range(0.0, 2.0 * PI);
	let dir_div = dir_1 * theta.sin() * theta2.cos() + dir_2 * theta.sin() * theta2.sin();
	let dirf = dir * theta.cos() + dir_div;
	let v_out = dirf * v_mag;
	v_out
}

pub enum OvenAperture {
	Cubic { size: [f64; 3] },
	Circular { radius: f64, thickness: f64 },
}

/// Component representing an oven, which is a source of hot atoms.
pub struct Oven {
	/// Temperature of the oven, in Kelvin
	pub temperature: f64,

	/// Size of the oven's aperture, SI units of metres.
	pub aperture: OvenAperture,

	/// A vector denoting the direction of the oven.
	pub direction: Vector3<f64>,
}

impl Component for Oven {
	type Storage = HashMapStorage<Self>;
}
impl Oven {
	pub fn get_random_spawn_position(&self) -> Vector3<f64> {
		let mut rng = rand::thread_rng();
		match self.aperture {
			OvenAperture::Cubic { size } => {
				let size = size.clone();
				let pos1 = rng.gen_range(-0.5 * size[0], 0.5 * size[0]);
				let pos2 = rng.gen_range(-0.5 * size[1], 0.5 * size[1]);
				let pos3 = rng.gen_range(-0.5 * size[2], 0.5 * size[2]);
				Vector3::new(pos1, pos2, pos3)
			}
			OvenAperture::Circular { radius, thickness } => {
				let dir = self.direction.normalize();
				let dir_1 = dir.cross(&Vector3::new(2.0, 1.0, 0.5)).normalize();
				let dir_2 = dir.cross(&dir_1).normalize();
				let theta = rng.gen_range(0., 2. * constant::PI);
				let r = rng.gen_range(0., radius);
				let h = rng.gen_range(-0.5 * thickness, 0.5 * thickness);
				dir * h + r * dir_1 * theta.sin() + r * dir_2 * theta.cos()
			}
		}
	}
}

/// Caps the maximum velocity of atoms created by an oven.
///
/// This resource indicates that any atoms emitted from an oven with velocity greater than the cap should be destroyed.
/// To use the cap, add it as a resource to the world.
pub struct OvenVelocityCap {
	/// The maximum speed of an atom emitted by an oven. See [Velocity](struct.Velocity.html) for units.
	pub cap: f64,
}

/// This system creates atoms from an oven source.
///
/// The oven points in the direction [Oven.direction].
pub struct OvenCreateAtomsSystem;

impl<'a> System<'a> for OvenCreateAtomsSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, Oven>,
		ReadStorage<'a, AtomInfo>,
		ReadStorage<'a, AtomNumberToEmit>,
		ReadStorage<'a, Position>,
		ReadStorage<'a, MassDistribution>,
		Option<Read<'a, OvenVelocityCap>>,
		Read<'a, LazyUpdate>,
	);

	fn run(
		&mut self,
		(entities, oven, atom, numbers_to_emit, pos, mass_dist, velocity_cap, updater): Self::SystemData,
	) {
		let max_vel = match velocity_cap {
			Some(cap) => cap.cap,
			None => std::f64::MAX
		};

		for (oven, atom, number_to_emit, oven_position, mass_dist) in
			(&oven, &atom, &numbers_to_emit, &pos, &mass_dist).join()
		{
			for _i in 0..number_to_emit.number {
				let mass = mass_dist.draw_random_mass().value;
				let speed = maths::maxwell_generate(oven.temperature, mass);
				if speed > max_vel {
					continue;
				}

				let new_atom = entities.create();
				let new_vel = velocity_generate(speed, &oven.direction);
				let start_position = oven_position.pos + oven.get_random_spawn_position();
				updater.insert(
					new_atom,
					Position {
						pos: start_position,
					},
				);
				updater.insert(
					new_atom,
					Velocity {
						vel: new_vel.clone(),
					},
				);

				updater.insert(new_atom, Force::new());
				updater.insert(new_atom, Mass { value: mass });
				updater.insert(
					new_atom,
					AtomInfo {
						mup: atom.mup,
						muz: atom.muz,
						mum: atom.mum,
						frequency: atom.frequency,
						linewidth: atom.linewidth,
						saturation_intensity: atom.saturation_intensity,
					},
				);
				updater.insert(new_atom, Atom);
				updater.insert(new_atom, InitialVelocity { vel: new_vel });
				updater.insert(new_atom, NewlyCreated);
			}
		}
	}
}
