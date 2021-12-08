//! Type of atom source used for releasing (metal) atoms into a gaseous phase

extern crate nalgebra;

use super::emit::AtomNumberToEmit;
use super::precalc::{MaxwellBoltzmannSource, PrecalculatedSpeciesInformation};
use crate::constant;
use crate::constant::PI;
use crate::initiate::*;

use super::VelocityCap;
use super::WeightedProbabilityDistribution;
use rand;
use rand::distributions::Distribution;
use rand::Rng;

extern crate specs;
use crate::atom::*;
use nalgebra::Vector3;

use specs::{Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System};

fn velocity_generate(
	v_mag: f64,
	new_dir: &Vector3<f64>,
	theta_distribution: &WeightedProbabilityDistribution,
) -> (Vector3<f64>, f64) {
	let dir = &new_dir.normalize();
	let dir_1 = new_dir.cross(&Vector3::new(2.0, 1.0, 0.5)).normalize();
	let dir_2 = new_dir.cross(&dir_1).normalize();
	let mut rng = rand::thread_rng();
	let theta = theta_distribution.sample(&mut rng);
	let phi = rng.gen_range(0.0..2.0 * PI);
	let dir_div = dir_1 * theta.sin() * phi.cos() + dir_2 * theta.sin() * phi.sin();
	let dirf = dir * theta.cos() + dir_div;
	let v_out = dirf * v_mag;
	(v_out, theta)
}
/// Opening aperture of the oven
#[derive(Copy, Clone)]
pub enum OvenAperture {
	Cubic { size: [f64; 3] },
	Circular { radius: f64, thickness: f64 },
}

/// Builder struct for creating Ovens.
pub struct OvenBuilder {
	temperature: f64,
	aperture: OvenAperture,
	direction: Vector3<f64>,
	microchannel_radius: f64,
	microchannel_length: f64,
	max_theta: f64,
}
impl OvenBuilder {
	pub fn new(temperature_kelvin: f64, direction: Vector3<f64>) -> Self {
		Self {
			temperature: temperature_kelvin,
			aperture: OvenAperture::Circular {
				radius: 3.0e-3,
				thickness: 1.0e-3,
			},
			direction: direction.normalize(),
			microchannel_length: 4e-3,
			microchannel_radius: 0.2e-3,
			max_theta: PI / 2.0,
		}
	}

	pub fn with_microchannels(
		&mut self,
		microchannel_length: f64,
		microchannel_radius: f64,
	) -> &mut Self {
		self.microchannel_length = microchannel_length;
		self.microchannel_radius = microchannel_radius;
		self
	}

	pub fn with_lip(&mut self, lip_length: f64, lip_radius: f64) -> &mut Self {
		self.max_theta = (lip_radius / lip_length).atan();
		self
	}

	pub fn with_aperture(&mut self, aperture: OvenAperture) -> &mut Self {
		self.aperture = aperture;
		self
	}

	pub fn build(&self) -> Oven {
		Oven {
			temperature: self.temperature,
			aperture: self.aperture,
			direction: self.direction.normalize(),
			theta_distribution: create_jtheta_distribution(
				self.microchannel_radius,
				self.microchannel_length,
			),
			max_theta: self.max_theta,
		}
	}
}

/// Component representing an oven, which is a source of hot atoms.
///
/// # The structure of the oven:
/// The oven consists of
/// * An oven aperture, within which atoms are spawned.
/// * A direction, which defines the axis of the oven.
/// * A temperature, which characterises the outgoing velocity distribution
///
/// Atoms are emitted from the oven according to an angular distribution in the polar angle theta, where theta=0 coincides with the direction of the oven.
/// The angular distribution follows the j(theta) distribution, and is determined by the geometry of microchannels in the oven aperture.
/// Additionally, any atom spawned with an angle greater than `max_theta` is ignored.
/// For real ovens, the maximum theta is determined by geometric constraints, for example the presence of a 'lip' of given length and
/// aperture radius.
pub struct Oven {
	/// Temperature of the oven, in Kelvin
	pub temperature: f64,

	/// Size of the oven's aperture, SI units of metres.
	pub aperture: OvenAperture,

	/// A vector denoting the direction of the oven.
	pub direction: Vector3<f64>,

	/// Angular distribution for atoms emitted by the oven.
	theta_distribution: WeightedProbabilityDistribution,

	/// The maximum angle theta at which atoms can be emitted from the oven. This can be constricted eg by a heat shield, or 'hot lip'.
	pub max_theta: f64,
}
impl MaxwellBoltzmannSource for Oven {
	fn get_temperature(&self) -> f64 {
		self.temperature
	}
	fn get_v_dist_power(&self) -> f64 {
		3.0
	}
}
impl Component for Oven {
	type Storage = HashMapStorage<Self>;
}
impl Oven {
	pub fn get_random_spawn_position(&self) -> Vector3<f64> {
		let mut rng = rand::thread_rng();
		match self.aperture {
			OvenAperture::Cubic { size } => {
				let size = size;
				let pos1 = rng.gen_range(-0.5 * size[0]..0.5 * size[0]);
				let pos2 = rng.gen_range(-0.5 * size[1]..0.5 * size[1]);
				let pos3 = rng.gen_range(-0.5 * size[2]..0.5 * size[2]);
				Vector3::new(pos1, pos2, pos3)
			}
			OvenAperture::Circular { radius, thickness } => {
				let dir = self.direction.normalize();
				let dir_1 = dir.cross(&Vector3::new(2.0, 1.0, 0.5)).normalize();
				let dir_2 = dir.cross(&dir_1).normalize();
				let theta = rng.gen_range(0.0..2. * constant::PI);
				let r = rng.gen_range(0.0..radius);
				let h = rng.gen_range(-0.5 * thickness..0.5 * thickness);
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
		ReadStorage<'a, AtomicTransition>,
		ReadStorage<'a, AtomNumberToEmit>,
		ReadStorage<'a, Position>,
		ReadStorage<'a, PrecalculatedSpeciesInformation>,
		Option<Read<'a, VelocityCap>>,
		Read<'a, LazyUpdate>,
	);

	fn run(
		&mut self,
		(entities, oven, atom, numbers_to_emit, pos, precalcs, velocity_cap, updater): Self::SystemData,
	) {
		let max_vel = match velocity_cap {
			Some(cap) => cap.value,
			None => std::f64::MAX,
		};

		let mut rng = rand::thread_rng();
		for (oven, atom, number_to_emit, oven_position, precalcs) in
			(&oven, &atom, &numbers_to_emit, &pos, &precalcs).join()
		{
			for _i in 0..number_to_emit.number {
				let (mass, speed) = precalcs.generate_random_mass_v(&mut rng);
				if speed > max_vel {
					continue;
				}

				let new_atom = entities.create();
				let (new_vel, theta) =
					velocity_generate(speed, &oven.direction, &oven.theta_distribution);

				if theta > oven.max_theta {
					continue;
				}
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
						vel: new_vel,
					},
				);
				updater.insert(new_atom, Force::new());
				updater.insert(new_atom, Mass { value: mass });
				updater.insert(new_atom, *atom);
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
				+ 2.0 / (3.0 * q) * (1.0 - 2.0 * alpha) * (1.0 - (1.0 - q.powf(2.0)).powf(1.5)))
	// (4.21)
	} else {
		j_theta = alpha * theta.cos() + 4.0 / (3.0 * PI * q) * (1.0 - 2.0 * alpha) * theta.cos();
		// (4.22)
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

	
	WeightedProbabilityDistribution::new(thetas, weights)
}
