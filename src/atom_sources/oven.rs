use crate::maths;
extern crate nalgebra;
extern crate rand;
use super::mass::MassArchetype;
use crate::constant;
use crate::constant::PI;
use crate::initiate::*;
use crate::integrator::Timestep;
use rand::Rng;

extern crate specs;
use crate::atom::*;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use specs::{
	Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System,
	WriteStorage,
};

pub fn velocity_generate(t: f64, mass: f64, new_dir: &Vector3<f64>) -> Vector3<f64> {
	let v_mag = maths::maxwell_generate(t, mass);
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
		ReadStorage<'a, MassArchetype>,
		Read<'a, LazyUpdate>,
	);

	fn run(
		&mut self,
		(entities, oven, atom, numbers_to_emit, pos, masstype, updater): Self::SystemData,
	) {
		for (oven, atom, number_to_emit, oven_position, masstype) in
			(&oven, &atom, &numbers_to_emit, &pos, &masstype).join()
		{
			for _i in 0..number_to_emit.number {
				let mass = masstype.get_mass().value;
				let new_atom = entities.create();
				let new_vel =
					velocity_generate(oven.temperature, mass * constant::AMU, &oven.direction);
				let start_position = oven_position.pos + oven.get_random_spawn_position();
				updater.insert(
					new_atom,
					Position {
						pos: start_position,
					},
				);
				updater.insert(new_atom, Velocity { vel: new_vel });
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
				updater.insert(new_atom, NewlyCreated);
			}
		}
	}
}

/// Component which indicates the oven should emit a number of atoms per frame.
#[derive(Serialize, Deserialize, Clone)]
pub struct EmitNumberPerFrame {
	pub number: i32,
}
impl Component for EmitNumberPerFrame {
	type Storage = HashMapStorage<Self>;
}

/// Component which indicates the oven should emit at a fixed average rate.
#[derive(Serialize, Deserialize, Clone)]
pub struct EmitFixedRate {
	pub rate: f64,
}
impl Component for EmitFixedRate {
	type Storage = HashMapStorage<Self>;
}

/// The number of atoms the oven should emit in the current frame.
pub struct AtomNumberToEmit {
	pub number: i32,
}
impl Component for AtomNumberToEmit {
	type Storage = HashMapStorage<Self>;
}

/// Calculates the number of atoms to emit per frame for fixed atoms-per-timestep ovens
pub struct EmitNumberPerFrameSystem;
impl<'a> System<'a> for EmitNumberPerFrameSystem {
	type SystemData = (
		ReadStorage<'a, EmitNumberPerFrame>,
		WriteStorage<'a, AtomNumberToEmit>,
	);

	fn run(&mut self, (emit_numbers, mut numbers_to_emit): Self::SystemData) {
		for (emit_number, mut number_to_emit) in (&emit_numbers, &mut numbers_to_emit).join() {
			number_to_emit.number = emit_number.number;
		}
	}
}

/// Calculates the number of atoms to emit each frame for sources with a fixed rate of emission.
///
/// There may be some random fluctuations in the numbers emitted each frame when the ratio of rate
/// and timestep duration is not an integer. The average rate will be correct.
pub struct EmitFixedRateSystem;
impl<'a> System<'a> for EmitFixedRateSystem {
	type SystemData = (
		ReadStorage<'a, EmitFixedRate>,
		ReadExpect<'a, Timestep>,
		WriteStorage<'a, AtomNumberToEmit>,
	);

	fn run(&mut self, (rates, timestep, mut emit_numbers): Self::SystemData) {
		let mut rng = rand::thread_rng();
		for (rate, mut emit_numbers) in (&rates, &mut emit_numbers).join() {
			let avg_number_to_emit = rate.rate * timestep.delta;
			let guaranteed_number = avg_number_to_emit.floor();
			let number: i32;
			if rng.next_f64() < avg_number_to_emit - guaranteed_number {
				number = guaranteed_number as i32 + 1;
			} else {
				number = guaranteed_number as i32;
			}
			emit_numbers.number = number;
		}
	}
}
