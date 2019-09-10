use crate::maths;
extern crate nalgebra;

use super::emit::AtomNumberToEmit;
use super::mass::MassDistribution;
use crate::constant;
use crate::constant::{PI,BOLTZCONST,EXP};
use crate::initiate::*;

extern crate rand;
use rand::Rng;
use super::WeightedProbabilityDistribution;
use rand::distributions::Distribution;

extern crate specs;
use crate::atom::*;
use nalgebra::Vector3;

use specs::{Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System};

fn velocity_generate(
	v_mag: f64,
	new_dir: &Vector3<f64>,
	theta_distribution: &WeightedProbabilityDistribution,
) -> Vector3<f64> {
	let dir = &new_dir.normalize();
	let dir_1 = new_dir.cross(&Vector3::new(2.0, 1.0, 0.5)).normalize();
	let dir_2 = new_dir.cross(&dir_1).normalize();
	let mut rng = rand::thread_rng();
	let theta = theta_distribution.sample(&mut rng);
	let phi = rng.gen_range(0.0, 2.0 * PI);
	let dir_div = dir_1 * theta.sin() * phi.cos() + dir_2 * theta.sin() * phi.sin();
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

	/// Angular distribution for atoms emitted by the oven.
	theta_distribution: WeightedProbabilityDistribution,
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

	pub fn new(temperature: f64, aperture: OvenAperture, direction: Vector3<f64>) -> Self {
		Oven {
			temperature: temperature,
			aperture: aperture,
			direction: direction.normalize(),
			theta_distribution: create_jtheta_distribution(0.2e-3, 4.0e-3),
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
			None => std::f64::MAX,
		};

		for (oven, atom, number_to_emit, oven_position, mass_dist) in
			(&oven, &atom, &numbers_to_emit, &pos, &mass_dist).join()
		{
			for _i in 0..number_to_emit.number {
				let mass = mass_dist.draw_random_mass().value;
				let speed = maths::maxwell_generate(oven.temperature, constant::AMU * mass);
				if speed > max_vel {
					continue;
				}

				let new_atom = entities.create();
				let new_vel = velocity_generate(speed, &oven.direction, &oven.theta_distribution);
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
				updater.insert(new_atom, atom.clone());
				updater.insert(new_atom, Atom);
				updater.insert(new_atom, InitialVelocity { vel: new_vel });
				updater.insert(new_atom, NewlyCreated);
			}
		}
	}
}

/// The jtheta distribution describes the angular dependence of atoms emitted from an oven.
/// It describes collision-free flow through a cylindrical channel (transparent mode of
/// operation).
///
/// See the book _Atomic and Molecular Beam Methods_, Vol. I, Scoles et al. The jtheta
/// distribution is defined on p88 in Section 4.2.2.1. Equation numbers below refer to
/// those in this reference.
///
/// Note that j(theta) is the defined with respect to solid angle, `Omega`. It is independent
/// of polar coordinate `phi`. The proportion of atoms going into the solid angle `Omega` at polar
/// coordinates `theta`, `phi` is proportional to `j(theta,phi)`.
///
/// The total solid angle presented by polar angle `theta` varies as the Jacobian `2 pi sin(theta)`.
/// Thus, the proportion of atoms with polar angle between `theta` and `theta + d_theta` is given by
/// `j(theta) 2 pi sin(theta) d_theta` - this is as above, but having _integrated out_ the coordinate
/// `phi`.
///
/// # Arguments
///
/// `theta`: The angle from the direction of the oven nozzle, in radians.
///
/// `channel_radius`: The radius of the cylindrical channels in the oven nozzle, m.
///
/// `channel_length`: The length of the cylindrical channels in the oven nozzle, m.
///
pub fn jtheta(theta: f64, channel_radius: f64, channel_length: f64) -> f64 {
	let beta = 2.0 * channel_radius / channel_length; // (4.16)
	let q = theta.tan() / beta; // (4.19)
	let alpha = 0.5 // (4.16)
		- 1.0 / (3.0 * beta.powf(2.0))
			* (1.0 - 2.0 * beta.powf(3.0)
				+ (2.0 * beta.powf(2.0) - 1.0) * (1.0 + beta.powf(2.0)).powf(0.5))
			/ ((1.0 + beta.powf(2.0)).powf(0.5) - beta.powf(2.0) * (1.0 / beta).asinh());

	let j_theta;
	if q <= 1.0 {
		let r_q = q.acos() - q * (1.0 - q.powf(2.0)).powf(0.5); // (4.23)
		j_theta = alpha * theta.cos()
			+ (2.0 / PI)
				* theta.cos() * ((1.0 - alpha) * r_q
				+ 2.0 / (3.0 * q) * (1.0 - 2.0 * alpha) * (1.0 - (1.0 - q.powf(2.0)).powf(1.5))) // (4.21)
	} else {
		j_theta = alpha * theta.cos() + 4.0 / (3.0 * PI * q) * (1.0 - 2.0 * alpha) * theta.cos(); // (4.22)
	}
	j_theta
}

/// Creates and precalculates a [WeightedIndex](struct.WeightedIndex.html) distribution
/// which can be used to sample values of theta based on the distribution
/// `p(theta) = j(theta) * sin(theta) * d_theta`.
///
/// The `j(theta)` function is discretised to use the
/// [WeightedIndex](struct.WeightedIndex.html).
fn create_jtheta_distribution(
	channel_radius: f64,
	channel_length: f64,
) -> WeightedProbabilityDistribution {
	// tuple list of (theta, weight)
	let mut thetas = Vec::<f64>::new();
	let mut weights = Vec::<f64>::new();

	// precalculate the discretized jtheta distribution.
	let n = 1000; // resolution over which to discretize `theta`.
	for i in 0..n {
		let theta = (i as f64 + 0.5) / (n as f64 + 1.0) * PI / 2.0;
		let weight = jtheta(theta, channel_radius, channel_length) * theta.sin();
		thetas.push(theta);
		weights.push(weight);
		// Note: we can exclude d_theta because it is constant and the distribution will be normalized.
	}

	let distribution = WeightedProbabilityDistribution::new(thetas, weights);
	distribution
}

/// The probability distribution `p(v)` that a given `mass` has a velocity magnitude `v`.
/// 
/// # Arguments
/// 
/// `temperature`: temperature of the gas, in Kelvin.
/// 
/// `mass`: particle mass, in SI units of kg. 
/// 
/// `v`: velocity magnitude, in SI units of m/s. 
/// 
pub fn maxwell_boltzmann_distribution(temperature: f64, mass: f64, v: f64) -> f64 {
	(mass / (2.0 * PI * BOLTZCONST * temperature)).powf(1.5)
		* EXP.powf(-mass * v.powf(2.0) / (2.0 * BOLTZCONST * temperature))
		* 4.0 * PI
		* v.powf(2.0)
}

/// Creates and precalculates a [WeightedProbabilityDistribution](struct.WeightedProbabilityDistribution.html)
/// which can be used to sample values of velocity, based on the Maxwell-Boltzmann distribution.
/// 
/// # Arguments
/// 
/// `temperature`: The temperature of the oven, in units of Kelvin.
/// 
/// `mass`: The mass of the particle, in SI units of kg.
fn create_v_distribution(
	temperature: f64,
	mass: f64,
) -> WeightedProbabilityDistribution {
	let max_velocity = 5.0 * (2.0 * BOLTZCONST * temperature / mass).powf(0.5);

	// tuple list of (velocity, weight)
	let mut velocities = Vec::<f64>::new();
	let mut weights = Vec::<f64>::new();

	// precalculate the discretized distribution.
	let n = 1000;
	for i in 0..n {
		let v = (i as f64 + 0.5) / (n as f64 + 1.0) * max_velocity;
		let weight = maxwell_boltzmann_distribution(temperature, mass, v);
		velocities.push(v);
		weights.push(weight);
	}

	WeightedProbabilityDistribution::new(velocities, weights)
}