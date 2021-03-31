//! Gaussian beam intensity distribution

extern crate nalgebra;
extern crate rayon;
extern crate specs;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage};

use crate::atom::Position;
use crate::maths;
use serde::{Deserialize, Serialize};

/// A component representing an intensity distribution with a gaussian profile.
///
/// The beam will propagate in vacuum. Inhomogenous media, gravitational lensing, refractions and
/// reflections (other than through a `CircularMask` are not implemented.
///
/// Also, attenuation effects are not yet implemented but they might come in a version
/// that accounts for atom-atom intereactions in the future.
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
impl GaussianBeam {
	/// Create a GaussianBeam component by specifying the peak intensity, rather than power.
	///
	/// # Arguments:
	///
	/// `intersection`: as per component.
	///
	/// `direction`: as per component.
	///
	/// `peak_intensity`: peak intensity in units of W/m^2.
	///
	/// `e_radius`: radius of beam in units of m.
	pub fn from_peak_intensity(
		intersection: Vector3<f64>,
		direction: Vector3<f64>,
		peak_intensity: f64,
		e_radius: f64,
	) -> Self {
		let std = e_radius / 2.0_f64.powf(0.5);
		let power = 2.0 * std::f64::consts::PI * std.powi(2) * peak_intensity;
		GaussianBeam {
			intersection: intersection,
			direction: direction,
			power: power,
			e_radius: e_radius,
		}
	}
}

/// A component that covers the central portion of a laser beam.
///
/// The mask is assumed to be coaxial to the GaussianBeam.
#[derive(Clone, Copy)]
pub struct CircularMask {
	/// Radius of the masked region in units of m.
	pub radius: f64,
}
impl Component for CircularMask {
	type Storage = HashMapStorage<Self>;
}

/// Returns the intensity of a gaussian laser beam at the specified position.
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
	use crate::constant::PI;
	use assert_approx_eq::assert_approx_eq;

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
}
