extern crate specs;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use super::cooling::{CoolingLight, CoolingLightIndex};
use super::sampler::LaserSamplers;
use crate::atom::Position;
use crate::maths;

/// A component representing a beam with a gaussian intensity profile.
pub struct GaussianBeam {
	/// A point that the laser beam intersects
	pub intersection: [f64; 3],

	/// Direction the beam propagates with respect to cartesian `x,y,z` axes.
	pub direction: [f64; 3],

	/// Radius of the beam at which the intensity is 1/e of the peak value, SI units of m.
	pub e_radius: f64,

	/// Power of the laser in W
	pub power: f64,
}
impl Component for GaussianBeam {
	type Storage = HashMapStorage<Self>;
}

/// System that calculates that samples the intensity of `GaussianBeam` entities.
pub struct SampleGaussianBeamIntensitySystem;
impl<'a> System<'a> for SampleGaussianBeamIntensitySystem {
	type SystemData = (
		ReadStorage<'a, CoolingLight>,
		ReadStorage<'a, CoolingLightIndex>,
		ReadStorage<'a, GaussianBeam>,
		WriteStorage<'a, LaserSamplers>,
		ReadStorage<'a, Position>,
	);
	fn run(&mut self, (cooling, indices, gaussian, mut samplers, positions): Self::SystemData) {
		for (_, index, gaussian) in (&cooling, &indices, &gaussian).join() {
			for (sampler, pos) in (&mut samplers, &positions).join() {
				sampler.contents[index.index].intensity =
					get_gaussian_beam_intensity(&gaussian, &pos);
			}
		}
	}
}

/// Gets the intensity of a gaussian laser beam at the specified position.
fn get_gaussian_beam_intensity(beam: &GaussianBeam, pos: &Position) -> f64 {
	beam.power
		* maths::gaussian_dis(
			beam.e_radius / 2.0_f64.powf(0.5),
			maths::get_minimum_distance_line_point(&pos.pos, &beam.intersection, &beam.direction),
		)
}

#[cfg(test)]
pub mod tests {

	use super::*;

	extern crate specs;
	use crate::constant::PI;
	use assert_approx_eq::assert_approx_eq;
	use specs::{Builder, RunNow, World};

	#[test]
	fn test_get_gaussian_beam_intensity() {
		let beam = GaussianBeam {
			direction: [1.0, 0.0, 0.0],
			intersection: [0.0, 0.0, 0.0],
			e_radius: 2.0,
			power: 1.0,
		};

		let pos1 = Position {
			pos: [1.0, 0.0, 0.0],
		};
		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius),
			get_gaussian_beam_intensity(&beam, &pos1),
			1e-6_f64
		);

		let pos2 = Position {
			pos: [0.0, 1.0, 0.0],
		};
		assert_approx_eq!(
			1.0 / (PI.powf(0.5) * beam.e_radius) * (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
			get_gaussian_beam_intensity(&beam, &pos2),
			1e-6_f64
		);
	}
}
