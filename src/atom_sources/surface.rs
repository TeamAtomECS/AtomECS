extern crate nalgebra;

use super::emit::AtomNumberToEmit;
extern crate rand;
use super::VelocityCap;

use crate::atom::*;
use crate::initiate::NewlyCreated;
use crate::shapes::{Surface,Cylinder};

extern crate specs;
use specs::{
	Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System
};

pub struct SurfaceSource {
	/// The temperature of the surface source, in Kelvin.
	pub temperature: f64,
}
impl Component for SurfaceSource {
    type Storage = HashMapStorage<Self>;
}

/// This system creates atoms from an oven source.
///
/// The oven points in the direction [Oven.direction].
pub struct CreateAtomsOnSurfaceSystem;

impl<'a> System<'a> for CreateAtomsOnSurfaceSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, SurfaceSource>,
        ReadStorage<'a, Cylinder>,
		ReadStorage<'a, AtomInfo>,
		ReadStorage<'a, AtomNumberToEmit>,
		ReadStorage<'a, Position>,
		ReadStorage<'a, Mass>,
		Option<Read<'a, VelocityCap>>,
		Read<'a, LazyUpdate>,
	);

	fn run(
		&mut self,
		(entities, surfaces, shapes, atom_infos, numbers_to_emit, source_positions, masses, velocity_cap, updater): Self::SystemData,
	) {
        // obey velocity cap.
		let max_vel = match velocity_cap {
			Some(cap) => cap.value,
			None => std::f64::MAX,
		};

		let mut rng = rand::thread_rng();
		for (surface, shape, atom_info, number_to_emit, source_position, mass) in
			(&surfaces, &shapes, &atom_infos, &numbers_to_emit, &source_positions, &masses).join()
		{
			for _i in 0..number_to_emit.number {

                // generate a random position on the surface.
                let (position, normal) = shape.get_random_point_on_surface(&source_position.pos);

                // todo: lambert cosine
                let direction = -normal.normalize();

				// todo: generate random speed
                let speed = 1.0;

				if speed > max_vel {
					continue;
				}

                let velocity = speed * direction;

				let new_atom = entities.create();
				updater.insert(
					new_atom,
					Position {
						pos: position,
					},
				);
				updater.insert(
					new_atom,
					Velocity {
						vel: velocity,
					},
				);
				updater.insert(new_atom, Force::new());
				updater.insert(new_atom, mass.clone());
				updater.insert(new_atom, atom_info.clone());
				updater.insert(new_atom, Atom);
				updater.insert(new_atom, InitialVelocity { vel: velocity });
				updater.insert(new_atom, NewlyCreated);
			}
		}
	}
}