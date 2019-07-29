extern crate specs;
use specs::{Component,VecStorage};

pub mod atom_create;

pub struct Step{
	pub n : u64,
}

pub struct Timestep{
	pub t:f64,
}

pub struct AtomInfo{
	pub mup:f64,
	pub mum:f64,
	
	pub muz:f64,
	pub frequency:f64,
	pub gamma:f64,
}

impl Component for AtomInfo{
	type Storage = VecStorage<Self>;	
}