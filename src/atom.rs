extern crate nalgebra;
extern crate specs;
extern crate specs_derive;
use specs::{Component, NullStorage, VecStorage};
use std::ops::Add;
use nalgebra::Vector3;

/// Position of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres)
pub struct Position {
	pub pos: Vector3<f64>,
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
			force: self.force + other.force
		}
	}
}
impl Force {
	pub fn new() -> Self { Force { force: Vector3::new(0.0,0.0,0.0)}}
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
