extern crate nalgebra;
extern crate specs;
extern crate specs_derive;
use nalgebra::Vector3;
use specs::{Component, NullStorage, VecStorage};
use std::ops::Add;
use crate::constant::{C,BOHRMAG};

/// Position of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres)
pub struct Position {
	pub pos: Vector3<f64>,
}
impl Position {
	pub fn new() -> Self{ Position { pos: Vector3::new(0.0,0.0,0.0)}}
}

impl Component for Position {
	type Storage = VecStorage<Self>;
}

/// Velocity of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres/second)
pub struct Velocity {
	pub vel: Vector3<f64>,
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

/// Force applies to an entity, with respect to cartesian x,y,z axes.
///
/// SI units (Newtons)
pub struct Force {
	pub force: Vector3<f64>,
}
impl Component for Force {
	type Storage = VecStorage<Self>;
}
impl Add<Force> for Force {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Force {
			force: self.force + other.force,
		}
	}
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
pub struct Mass {
	pub value: f64,
}

impl Component for Mass {
	type Storage = VecStorage<Self>;
}

/// Component that marks an entity as an [atom](struct.Atom.html).
/// This provides a simple way for systems to get only [atom](struct.Atom.html)s, even though non-atom entities may also share components, eg [position](struct.Position.html).
#[derive(Component)]
#[storage(NullStorage)]
pub struct Atom;

impl Default for Atom {
	fn default() -> Self {
		Atom {}
	}
}

pub struct AtomInfo {
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
}

impl Component for AtomInfo {
	type Storage = VecStorage<Self>;
}
impl AtomInfo {
	/// Creates an `AtomInfo` component populated with parameters for Rubidium.
	/// The parameters are taken from Daniel Steck's Data sheet on Rubidium-87.
	pub fn rubidium() -> Self {
		AtomInfo {
			mup: BOHRMAG,
			mum: -BOHRMAG,
			muz: 0.0,
			frequency: C / 780.0e-9,
			linewidth: 6.065e6,          // [Steck, Rubidium87]
			saturation_intensity: 16.69, // [Steck, Rubidium 87, D2 cycling transition]
		}
	}

	pub fn gamma(&self) -> f64 {
		self.linewidth * 2.0 * std::f64::consts::PI
	}
}
