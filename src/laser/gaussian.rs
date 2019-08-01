extern crate specs;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use super::cooling::{CoolingLight, CoolingLightIndex};
use crate::atom::Position;
use crate::maths;
use super::intensity::LaserSamplers;

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
			beam.e_radius * 2.0_f64.powf(0.5),
			maths::get_minimum_distance_line_point(&pos.pos, &beam.intersection, &beam.direction),
		)
}
