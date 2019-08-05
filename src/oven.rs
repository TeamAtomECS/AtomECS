use crate::maths;
extern crate nalgebra;
extern crate rand;
use crate::constant;
use crate::constant::PI;
use crate::initiate::*;
use rand::Rng;
extern crate specs;
use crate::atom::*;
use nalgebra::Vector3;

use specs::{
	Component, DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadStorage, System,
	VecStorage, World,
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

/// Component representing an oven, which is a source of hot atoms.
pub struct Oven {
	/// Temperature of the oven, in Kelvin
	pub temperature: f64,

	/// Size of the oven's aperture, SI units of metres.
	pub size: [f64; 3],

	/// A vector denoting the direction of the oven.
	pub direction: Vector3<f64>,

	/// Number of atoms output by the oven every time step
	pub number: u64,
}

impl Component for Oven {
	type Storage = VecStorage<Self>;
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
		ReadStorage<'a, Position>,
		Read<'a, LazyUpdate>,
	);

	fn run(&mut self, (entities, oven, atom, pos, updater): Self::SystemData) {
		let mut rng = rand::thread_rng();

		for (oven, atom, oven_position) in (&oven, &atom, &pos).join() {
			// This is only temporary. In future the atom source will have a MassDistribution component,
			// which will allow us to specify the mass distribution of created atoms. For example,
			// the natural abundancies of Sr or Rb, or an enriched source of Potassium. Leave as
			// 87 for now.
			let mass = 87.0;
			let size = oven.size.clone();
			for _i in 0..oven.number {
				let new_atom = entities.create();
				let new_vel =
					velocity_generate(oven.temperature, mass * constant::AMU, &oven.direction);
				let pos1 = rng.gen_range(-0.5 * size[0], 0.5 * size[0]);
				let pos2 = rng.gen_range(-0.5 * size[1], 0.5 * size[1]);
				let pos3 = rng.gen_range(-0.5 * size[2], 0.5 * size[2]);
				let start_position = oven_position.pos + Vector3::new(pos1, pos2, pos3);
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

/// Adds required systems to the dispatcher.
pub fn add_systems_to_dispatch(
	builder: DispatcherBuilder<'static, 'static>,
	deps: &[&str],
) -> DispatcherBuilder<'static, 'static> {
	builder.with(OvenCreateAtomsSystem, "", deps)
}

/// Registers resources required by the module to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<Oven>();
}
