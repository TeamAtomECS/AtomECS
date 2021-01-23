extern crate nalgebra;
extern crate rayon;
extern crate specs;
use nalgebra::Vector3;
use specs::{Component, Entities, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use super::cooling::{CoolingLight, CoolingLightIndex};
use super::sampler::LaserSamplers;
use crate::atom::Position;
use crate::maths;
use serde::{Deserialize, Serialize};

/// A component representing a beam with a gaussian intensity profile.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct GaussianBeam {
	/// A point that the laser beam intersects
	pub intersection: Vector3<f64>,

	/// Direction the beam propagates with respect to cartesian `x,y,z` axes.
	pub direction: Vector3<f64>,

	/// Radius of the beam at which the intensity is 1/e of the peak value, SI units of m.
	pub e_radius: f64,

	/// Power of the laser in W
	pub power: f64,
}
impl Component for GaussianBeam {
	type Storage = HashMapStorage<Self>;
}

/// A component that covers the central portion of a laser beam.
///
/// The mask is assumed to be Coaxial to the GaussianBeam.
#[derive(Clone, Copy)]
pub struct CircularMask {
	/// Radius of the masked region.
	pub radius: f64,
}
impl Component for CircularMask {
	type Storage = HashMapStorage<Self>;
}

const LASER_CACHE_SIZE: usize = 16;

/// System that calculates that samples the intensity of `GaussianBeam` entities.
pub struct SampleGaussianBeamIntensitySystem;
impl<'a> System<'a> for SampleGaussianBeamIntensitySystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, CoolingLight>,
		ReadStorage<'a, CoolingLightIndex>,
		ReadStorage<'a, GaussianBeam>,
		ReadStorage<'a, CircularMask>,
		WriteStorage<'a, LaserSamplers>,
		ReadStorage<'a, Position>,
	);

	fn run(
		&mut self,
		(entities, cooling, indices, gaussian, masks, mut samplers, positions): Self::SystemData,
	) {
		use rayon::prelude::*;
		use specs::ParJoin;

		// There are typically only a small number of lasers in a simulation.
		// For a speedup, cache the required components into thread memory,
		// so they can be distributed to parallel workers during the atom loop.
		type CachedLaser = (
			CoolingLight,
			CoolingLightIndex,
			GaussianBeam,
			Option<CircularMask>,
		);
		let laser_cache: Vec<CachedLaser> = (&entities, &cooling, &indices, &gaussian)
			.join()
			.map(|(laser_entity, cooling, index, gaussian)| {
				(
					cooling.clone(),
					index.clone(),
					gaussian.clone(),
					masks.get(laser_entity).cloned(),
				)
			})
			.collect();

		// Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
		for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
			let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
			let slice = &laser_cache[base_index..max_index];
			let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
			laser_array[..max_index].copy_from_slice(slice);
			let number_in_iteration = slice.len();

			(&mut samplers, &positions)
				.par_join()
				.for_each(|(sampler, pos)| {
					for i in 0..number_in_iteration {
						let (cooling, index, gaussian, mask) = laser_array[i];
						sampler.contents[index.index].intensity =
							get_gaussian_beam_intensity(&gaussian, &pos, mask.as_ref());
						sampler.contents[index.index].polarization = cooling.polarization.into();
						sampler.contents[index.index].wavevector =
							gaussian.direction.normalize() * cooling.wavenumber();
					}
				});
		}
	}
}

/// Gets the intensity of a gaussian laser beam at the specified position.
pub fn get_gaussian_beam_intensity(
	beam: &GaussianBeam,
	pos: &Position,
	mask: Option<&CircularMask>,
) -> f64 {
	let min_dist =
		maths::get_minimum_distance_line_point(&pos.pos, &beam.intersection, &beam.direction);
	let power = match mask {
		Some(mask) => {
			if min_dist < mask.radius {
				0.0
			} else {
				beam.power
			}
		}
		None => beam.power,
	};
	power * maths::gaussian_dis(beam.e_radius / 2.0_f64.powf(0.5), min_dist)
}

#[cfg(test)]
pub mod tests {

	use super::*;

	extern crate specs;
	use crate::constant::{EXP, PI};
	use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
	use crate::laser::sampler::{LaserSampler, LaserSamplers};
	use assert_approx_eq::assert_approx_eq;
	use specs::{Builder, RunNow, World};

	extern crate nalgebra;
	use nalgebra::Vector3;

	#[test]
	fn test_get_gaussian_beam_intensity() {
		let beam = GaussianBeam {
			direction: Vector3::x(),
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 2.0,
			power: 1.0,
		};

		let pos1 = Position { pos: Vector3::x() };
		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0),
			get_gaussian_beam_intensity(&beam, &pos1, None),
			1e-6_f64
		);

		let pos2 = Position { pos: Vector3::y() };
		assert_approx_eq!(
			1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
				* (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
			get_gaussian_beam_intensity(&beam, &pos2, None),
			1e-6_f64
		);
	}

	#[test]
	fn test_sample_gaussian_beam_system() {
		let mut test_world = World::new();
		test_world.register::<CoolingLightIndex>();
		test_world.register::<CoolingLight>();
		test_world.register::<GaussianBeam>();
		test_world.register::<Position>();
		test_world.register::<LaserSamplers>();
		test_world.register::<CircularMask>();

		let e_radius = 2.0;
		let power = 1.0;
		test_world
			.create_entity()
			.with(CoolingLight {
				polarization: 1,
				wavelength: 780e-9,
			})
			.with(CoolingLightIndex {
				index: 0,
				initiated: true,
			})
			.with(GaussianBeam {
				direction: Vector3::x(),
				intersection: Vector3::new(0.0, 0.0, 0.0),
				e_radius: e_radius,
				power: power,
			})
			.build();

		let sampler1 = test_world
			.create_entity()
			.with(Position {
				pos: Vector3::new(0.0, 0.0, 0.0),
			})
			.with(LaserSamplers {
				contents: vec![LaserSampler::default()],
			})
			.build();

		let sampler2 = test_world
			.create_entity()
			.with(Position {
				pos: Vector3::new(0.0, e_radius, 0.0),
			})
			.with(LaserSamplers {
				contents: vec![LaserSampler::default()],
			})
			.build();

		let mut system = SampleGaussianBeamIntensitySystem;
		system.run_now(&test_world.res);
		test_world.maintain();
		let sampler_storage = test_world.read_storage::<LaserSamplers>();

		// Peak intensity
		assert_approx_eq!(
			sampler_storage
				.get(sampler1)
				.expect("entity not found")
				.contents[0]
				.intensity,
			power / (PI.powf(0.5) * e_radius).powf(2.0)
		);

		// 1 over e intensity radius
		assert_approx_eq!(
			sampler_storage
				.get(sampler2)
				.expect("entity not found")
				.contents[0]
				.intensity,
			power / (PI.powf(0.5) * e_radius).powf(2.0) / EXP
		);
	}
}
