use crate::constant::BOLTZCONST;
use crate::constant::EXP;
use crate::constant::PI;
extern crate rand;
use rand::Rng;
extern crate nalgebra;
use nalgebra::Vector3;

/// Get miniminum distance between a point and a line.
///
/// # Arguments
///
/// `pos`: position of the point
///
/// `line_point`: a point on the line
///
/// `dir`: vector pointing along the line.
pub fn get_minimum_distance_line_point(
	pos: &Vector3<f64>,
	line_point: &Vector3<f64>,
	dir: &Vector3<f64>,
) -> f64 {
	let rela_cood = pos - line_point;
	let distance = (dir.cross(&rela_cood) / dir.norm()).norm();
	distance
}

pub fn gaussian_dis(std: f64, distance: f64) -> f64 {
	//checked
	1.0 / ((2.0 * PI).powf(0.5) * std) * EXP.powf(-distance.powf(2.0) / 2.0 / (std).powf(2.0))
}

pub fn maxwell_dis(_t: f64, _mass: f64, _velocity: f64) -> f64 {
	(_mass / 2.0 / PI / BOLTZCONST / _t).powf(1.5)
		* EXP.powf(-_mass * _velocity.powf(2.0) / 2.0 / BOLTZCONST / _t)
		* 4.0 * PI
		* _velocity.powf(2.0)
}

pub fn maxwell_generate(_t: f64, _mass: f64) -> f64 {
	// take about 20 times of the variance as range and do random uniform generation
	// use 1/1000 times of the real PDF so that the maxwell distribution is everywhere lower than the uniform one

	let range = 20.0 * (BOLTZCONST * _t / _mass).powf(0.5);
	let mut i = 0;
	loop {
		let mut rng = rand::thread_rng();
		i = i + 1;

		let result = rng.gen_range(0.0, range);
		let height = rng.gen_range(0.0, 1.0 / range);
		if maxwell_dis(_t, _mass, result) > height * 1000.0 {
			return result;
		}
	}
}

pub fn jtheta(theta: f64) -> f64 {
	//checked (against dongyang matlab code)
	// problem remained what is it?
	let r = 0.2e-3;
	let length = 4.0e-3;
	let beta = 2.0 * r / length;
	let q_theta = 1.0 / beta * theta.tan();
	let r_theta = q_theta.acos() - q_theta * (1.0 - q_theta.powf(2.0)).powf(0.5);
	let alpha = 1.0 / 2.0
		- 1.0 / (3.0 * beta.powf(2.0))
			* (1.0 - 2.0 * beta.powf(3.0)
				+ (2.0 * beta.powf(2.0) - 1.0) * (1.0 + beta.powf(2.0)).powf(0.5))
			/ ((1.0 + beta.powf(2.0)).powf(0.5) - beta.powf(2.0) * (1.0 / beta).asinh());
	let j1_theta = alpha * theta.cos()
		+ 2.0 / PI
			* theta.cos()
			* ((1.0 - alpha) * r_theta
				+ 2.0 / (3.0 * q_theta)
					* (1.0 - 2.0 * alpha)
					* (1.0 - (1.0 - q_theta.powf(2.0)).powf(3.0 / 2.0)));
	let j2_theta =
		alpha * theta.cos() + 4.0 / (3.0 * PI * q_theta) * (1.0 - 2.0 * alpha) * theta.cos();
	if q_theta < 1.0 {
		j1_theta * 2.0 * PI * theta.sin()
	} else {
		j2_theta * 2.0 * PI * theta.sin()
	}
}
pub fn jtheta_gen() -> f64 {
	//generate a random sample based on jtheta distribution
	// use 1/20 of the jetha function so that the uniform distribution is always above the jtheta
	let mut i = 0;
	loop {
		let mut rng = rand::thread_rng();
		i = i + 1;

		let result = rng.gen_range(0.0, PI / 2.0);
		let height = rng.gen_range(0.0, 2.0 / PI);
		if jtheta(result) > height * 20.0 {
			return result;
		}
	}
}
pub fn random_direction() -> Vector3<f64> {
	let mut rng = rand::thread_rng();
	let angle1 = rng.gen_range(0.0, PI);
	let angle2 = rng.gen_range(0., 2. * PI);
	let result = Vector3::new (
		angle1.cos(),
		angle1.sin() * angle2.sin(),
		angle1.sin() * angle2.cos(),
	);
	//println!("{:?}",result);
	result
}
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn distribution_test() {
		assert!(jtheta(1.) > 0.2174 && jtheta(1.) < 0.2176, "jtheta ");

		assert!(
			maxwell_dis(300., 1e-25, 100.) > 0.000839 && maxwell_dis(300., 1e-25, 100.) < 0.000840
		);
	}

	#[test]
	fn test_minimum_distance_line_point() {
		let pos = Vector3::new(1., 1., 1.);
		let centre = Vector3::new(0., 1., 1.);
		let dir = Vector3::new(1., 2., 2.);
		let distance = get_minimum_distance_line_point(&pos, &centre, &dir);
		assert!(distance > 0.942, distance < 0.943);
	}
}
