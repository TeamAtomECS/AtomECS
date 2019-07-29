extern crate specs;
use specs::{Component, VecStorage};

//this file is simply straightforward
pub struct Position{
	pub pos:[f64;3]
}

impl Component for Position{
	type Storage = VecStorage<Self>;
}

pub struct Velocity{
	pub vel:[f64;3]
}

impl Component for Velocity{
	type Storage = VecStorage<Self>;
}

pub struct Force{
	pub force:[f64;3]
}

impl Component for Force{
	type Storage = VecStorage<Self>;
}



pub struct RandKick{
	pub force:[f64;3]
}

impl Component for RandKick{
	type Storage = VecStorage<Self>;
}
