//! Common atom components and systems.

use crate::constant::{BOHRMAG, C};
use crate::output::file::BinaryConversion;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use specs::{Component, NullStorage, System, VecStorage, World, WriteStorage};
use std::fmt;

/// Position of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres)
#[derive(Deserialize, Serialize, Clone)]
pub struct Position {
	/// position in 3D in units of m
	pub pos: Vector3<f64>,
}
impl Position {
	pub fn new() -> Self {
		Position {
			/// position in 3D in units of m
			pos: Vector3::new(0.0, 0.0, 0.0),
		}
	}
}

impl Component for Position {
	type Storage = VecStorage<Self>;
}
impl fmt::Display for Position {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({:?},{:?},{:?})", self.pos[0], self.pos[1], self.pos[2])
	}
}
impl BinaryConversion for Position {
	fn data(&self) -> Vec<f64> {
		vec![self.pos[0], self.pos[1], self.pos[2]]
	}
}

/// Velocity of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres/second)
#[derive(Clone)]
pub struct Velocity {
	/// velocity vector in 3D in units of m/s
	pub vel: Vector3<f64>,
}
impl fmt::Display for Velocity {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({:?},{:?},{:?})", self.vel[0], self.vel[1], self.vel[2])
	}
}
impl BinaryConversion for Velocity {
	fn data(&self) -> Vec<f64> {
		vec![self.vel[0], self.vel[1], self.vel[2]]
	}
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

/// Initial velocity of an atom.
///
/// See [Velocity](struct.Velocity.html).
pub struct InitialVelocity {
	/// velocity vector in 3D in units of m/s
	pub vel: Vector3<f64>,
}
impl Component for InitialVelocity {
	type Storage = VecStorage<Self>;
}

/// Force applies to an entity, with respect to cartesian x,y,z axes.
///
/// SI units (Newtons)
#[derive(Copy, Clone)]
pub struct Force {
	/// force vector in 3D in units of N
	pub force: Vector3<f64>,
}
impl Component for Force {
	type Storage = VecStorage<Self>;
}
impl Force {
	pub fn new() -> Self {
		Force {
			force: Vector3::new(0.0, 0.0, 0.0),
		}
	}
}

/// Inertial and Gravitational mass of an entity
///
/// Mass is specified in atom mass units (amu).
#[derive(Deserialize, Serialize, Clone)]
pub struct Mass {
	/// mass value in atom mass units
	pub value: f64,
}

impl Component for Mass {
	type Storage = VecStorage<Self>;
}

/// Component that marks an entity as an [atom](struct.Atom.html).
/// This provides a simple way for systems to get only [atom](struct.Atom.html)s, even though non-atom entities may also share components, eg [position](struct.Position.html).
#[derive(Default)]
pub struct Atom;

impl Component for Atom {
	type Storage = NullStorage<Self>;
}

/// Helper enum to identify the kind of atomic transition later in match operators
/// Could be integrated in the Atom component, maybe.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum Kind {
	Rubidium,
	Strontium,
	StrontiumRed,
	Erbium,
	Erbium401,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct AtomicTransition {
	/// The dependence of the sigma_+ transition on magnetic fields.
	/// The sigma_+ transition is shifted by `mup * field.magnitude / h` Hz.
	/// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
	pub mup: f64,
	/// The dependence of the sigma_- transition on magnetic fields.
	/// The sigma_- transition is shifted by `mum * field.magnitude / h` Hz.
	/// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
	pub mum: f64,
	/// The dependence of the sigma_pi transition on magnetic fields.
	/// The sigma_pi transition is shifted by `muz * field.magnitude / h` Hz.
	/// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
	pub muz: f64,
	/// Frequency of the laser cooling transition, Hz.
	pub frequency: f64,
	/// Linewidth of the laser cooling transition, Hz
	pub linewidth: f64,
	/// Saturation intensity, in units of W/m^2.
	pub saturation_intensity: f64,
	/// Precalculate prefactor used in the determination of rate coefficients.
	pub rate_prefactor: f64,
	/// Nametag for match control operators to identify later on
	pub kind: Kind,
}

impl Component for AtomicTransition {
	type Storage = VecStorage<Self>;
}
impl AtomicTransition {
	pub fn calculate(mut self) -> Self {
		// self.rate_prefactor =
		// 	PI.powi(2) * C.powi(2) * (2.0 * PI * self.frequency).powi(-3) * self.gamma().powi(2)
		// 		/ HBAR;
		// For a pure two-level system, the saturation intensity is equal to (hbar omega^3 Gamma) / (8 pi^2 c^2)
		self.rate_prefactor = self.gamma().powi(3) / (self.saturation_intensity * 8.0);
		self
	}

	/// Creates an `AtomicTransition` component populated with parameters for Rubidium.
	/// The parameters are taken from Daniel Steck's Data sheet on Rubidium-87.
	pub fn rubidium() -> Self {
		AtomicTransition {
			mup: BOHRMAG,
			mum: -BOHRMAG,
			muz: 0.0,
			frequency: C / 780.0e-9,
			linewidth: 6.065e6,          // [Steck, Rubidium87]
			saturation_intensity: 16.69, // [Steck, Rubidium 87, D2 cycling transition]
			rate_prefactor: 0.0,         // set in calculate
			kind: Kind::Rubidium,
		}
		.calculate()
	}

	/// Creates an `AtomicTransition` component populated with parameters for Strontium.
	/// The parameters are taken from doi:10.1103/PhysRevA.97.039901 [Nosske 2017].
	pub fn strontium() -> Self {
		AtomicTransition {
			mup: BOHRMAG,  // to check, should be correct
			mum: -BOHRMAG, // to check
			muz: 0.0,
			frequency: 650_759_219_088_937.,
			linewidth: 32e6,             // [Nosske2017]
			saturation_intensity: 430.0, // [Nosske2017, 43mW/cm^2]
			rate_prefactor: 0.0,         // set in calculate
			kind: Kind::Strontium,
		}
		.calculate()
	}

	/// Creates an `AtomicTransition` component populated with parameters for red Strontium transition.
	/// The parameters are taken from NIST, doi:10.1063/1.344917 and Schreck2013.
	pub fn strontium_red() -> Self {
		AtomicTransition {
			mup: 3. / 2. * BOHRMAG,  // to check, yes seriously, I think it's 3/2*BOHRMAG!
			mum: -3. / 2. * BOHRMAG, // to check
			muz: 0.0,
			frequency: 434_829_121_311_000., // NIST, doi:10.1063/1.344917
			linewidth: 7_400.,               // [Schreck2013]
			saturation_intensity: 0.0295,    // [SChreck2013, 3 µW/cm^2]
			rate_prefactor: 0.0,             // set in calculate
			kind: Kind::StrontiumRed,
		}
		.calculate()
	}

	/// Creates an `AtomicTransition` component populated with parameters for Erbium.
	pub fn erbium() -> Self {
		AtomicTransition {
			mup: BOHRMAG,
			mum: -BOHRMAG,
			muz: 0.0,
			frequency: 5.142e14,
			linewidth: 190e3,
			saturation_intensity: 0.13,
			rate_prefactor: 0.0, // set in calculate
			kind: Kind::Erbium,
		}
		.calculate()
	}
	/// Creates an `AtomicTransition` component populated with parameters for Erbium 401 .
	pub fn erbium_401() -> Self {
		AtomicTransition {
			mup: 1.1372 * BOHRMAG,
			mum: 1.1372 * -BOHRMAG,
			muz: 0.0,
			frequency: 7.476e14,
			linewidth: 30e6,
			saturation_intensity: 56.0,
			rate_prefactor: 0.0, // set in calculate
			kind: Kind::Erbium401,
		}
		.calculate()
	}

	pub fn gamma(&self) -> f64 {
		self.linewidth * 2.0 * std::f64::consts::PI
	}
}

/// A system that sets force to zero at the start of each simulation step.
pub struct ClearForceSystem;

impl<'a> System<'a> for ClearForceSystem {
	type SystemData = WriteStorage<'a, Force>;
	fn run(&mut self, mut force: Self::SystemData) {
		use rayon::prelude::*;
		use specs::ParJoin;

		(&mut force).par_join().for_each(|force| {
			force.force = Vector3::new(0.0, 0.0, 0.0);
		});
	}
}

/// Registers resources required by `atom_sources` to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<Position>();
	world.register::<Mass>();
	world.register::<Force>();
	world.register::<AtomicTransition>();
	world.register::<Atom>();
	world.register::<InitialVelocity>();
	world.register::<Velocity>();
}
