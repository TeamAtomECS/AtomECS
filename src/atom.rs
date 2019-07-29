extern crate specs;
use specs::{Component, VecStorage};

/// Position of an entity in space, with respect to cartesian x,y,z axes. 
/// 
/// SI units (metres)
pub struct Position{
	pub pos:[f64;3]
}

impl Component for Position{
	type Storage = VecStorage<Self>;
}

/// Velocity of an entity in space, with respect to cartesian x,y,z axes.
/// 
/// SI units (metres/second)
pub struct Velocity{
	pub vel:[f64;3]
}

impl Component for Velocity{
	type Storage = VecStorage<Self>;
}

/// Force applies to an entity, with respect to cartesian x,y,z axes. 
/// 
/// SI units (Newtons)
pub struct Force{
	pub force:[f64;3]
}

impl Component for Force{
	type Storage = VecStorage<Self>;
}

/// Inertial and Gravitational mass of an entity
/// 
/// Mass is specified in atom mass units (amu).
pub struct Mass {
	pub value:f64
}

impl Component for Mass {
	type Storage = VecStorage<Self>;
}


pub struct RandKick{
	pub force:[f64;3]
}

impl Component for RandKick{
	type Storage = VecStorage<Self>;
}
