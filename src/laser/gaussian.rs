//! Gaussian beam intensity distribution

extern crate nalgebra;
extern crate rayon;
extern crate specs;
use nalgebra::Vector3;
use specs::VecStorage;
use specs::{Component, HashMapStorage};

use crate::atom::Position;
use crate::constant::EXP;
use crate::constant::PI;
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
	///
	/// Since in the literature the e^2_radius (where intensity is 1/e^2 of peak value) is used
	/// very often as well, it is useful to note the following relation:
	///
	/// e_radius = e^2_radius / sqrt(2)
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
	rayleigh: Option<&GaussianRayleighRange>,
) -> f64 {
	let (min_dist, z) =
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
	let broadening_factor = match rayleigh {
		Some(rayleigh_range) => 1. / (1. + z.powf(2.0) / rayleigh_range.rayleigh_range.powf(2.0)),
		None => 1.0,
	};
	power * broadening_factor * maths::gaussian_dis(beam.e_radius / 2.0_f64.powf(0.5), min_dist)
}

/// A component that enables the correct treatment of the `GaussianBeam` for cases where
/// it is strongly focused, i.e. the beam waist is very small compared to the axial
/// length on which the intensity is required.
///
/// This is especially important for the dipole force since the axial gradient is
/// crucial for optical transport. For most MOT simulations, this component is not
/// required since rayleigh ranges are typically several hundreds of metres.
#[derive(Clone, Copy)]
pub struct GaussianRayleighRange {
	/// The distance along the propagation direction of a beam from the
	///  waist to the place where the area of the cross section is doubled in units of metres
	pub rayleigh_range: f64,
}

/// Computes the rayleigh range for a given beam and wavelength
pub fn make_gaussian_rayleigh_range(
	wavelength: &f64,
	gaussian: &GaussianBeam,
) -> GaussianRayleighRange {
	GaussianRayleighRange {
		rayleigh_range: 2.0 * PI * gaussian.e_radius.powf(2.0) / wavelength,
	}
}

impl Component for GaussianRayleighRange {
	type Storage = VecStorage<Self>;
}

/// A component that stores additional information about a given beam
/// entity, such as internal reference frame and ellipticity
#[derive(Clone, Copy)]
pub struct GaussianReferenceFrame {
	pub x_vector: Vector3<f64>,
	pub y_vector: Vector3<f64>,
	pub ellipticity: f64,
}
impl Component for GaussianReferenceFrame {
	type Storage = VecStorage<Self>;
}

/// Computes the intensity gradient of a given beam with and returns it as
/// a three-dimensional vector
pub fn get_gaussian_beam_intensity_gradient(
	beam: &GaussianBeam,
	pos: &Position,
	rayleigh: &GaussianRayleighRange,
	reference_frame: &GaussianReferenceFrame,
) -> Vector3<f64> {
	let rela_coord = pos.pos - beam.intersection;
	let x = rela_coord.dot(&reference_frame.x_vector);
	let y = rela_coord.dot(&reference_frame.y_vector);
	let z = rela_coord.dot(&beam.direction);

	let spot_size_squared =
		2.0 * beam.e_radius.powf(2.0) * (1. + (z / rayleigh.rayleigh_range).powf(2.0));
	let vector = -4. * (reference_frame.x_vector * x + reference_frame.y_vector * y)
		+ beam.direction * z / (rayleigh.rayleigh_range.powf(2.0) + z.powf(2.0))
			* (-spot_size_squared + 4. * (x.powf(2.0) + y.powf(2.0)));
	let intensity = 2. * beam.power / PI / spot_size_squared
		* EXP.powf(-2. * (x.powf(2.0) + y.powf(2.0)) / spot_size_squared);

	intensity / spot_size_squared * vector
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
	fn test_get_gaussian_beam_intensity_gradient() {
		let beam = GaussianBeam {
			direction: Vector3::z(),
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 70.71067812e-6,
			power: 100.0,
		};
		let pos1 = Position {
			pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
		};
		let rayleigh_rng = make_gaussian_rayleigh_range(&1064.0e-9, &beam);
		let grf = GaussianReferenceFrame {
			x_vector: Vector3::x(),
			y_vector: Vector3::y(),
			ellipticity: 0.0,
		};

		let gradient = get_gaussian_beam_intensity_gradient(&beam, &pos1, &rayleigh_rng, &grf);
		assert_approx_eq!(gradient[0], -2.49605032e+13, 1e+8_f64);
		assert_approx_eq!(gradient[1], 0.0, 1e+9_f64);
		assert_approx_eq!(gradient[2], -2.06143366e+08, 1e+6_f64);
	}

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
			get_gaussian_beam_intensity(&beam, &pos1, None, None),
			1e-6_f64
		);

		let pos2 = Position { pos: Vector3::y() };
		assert_approx_eq!(
			1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
				* (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
			get_gaussian_beam_intensity(&beam, &pos2, None, None),
			1e-6_f64
		);

		let rayleigh_range = GaussianRayleighRange {
			rayleigh_range: 1.0,
		};

		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0) / 2.,
			get_gaussian_beam_intensity(&beam, &pos1, None, Some(&rayleigh_range)),
			1e-6_f64
		);

		assert_approx_eq!(
			1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
				* (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
			get_gaussian_beam_intensity(&beam, &pos2, None, Some(&rayleigh_range)),
			1e-6_f64
		);
		let rayleigh_range_2 = make_gaussian_rayleigh_range(&461.0e-6, &beam);

		let pos3 = Position {
			pos: Vector3::x() * rayleigh_range_2.rayleigh_range,
		};
		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0) / 2.,
			get_gaussian_beam_intensity(&beam, &pos3, None, Some(&rayleigh_range_2)),
			1e-6_f64
		);
	}
}
