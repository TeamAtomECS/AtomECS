extern crate specs;
use specs::{Component,VecStorage};

pub mod atom_create;

pub struct AtomInfo{
	pub mup:f64,
	pub mum:f64,
	pub mass:u64,
	pub muz:f64,
	pub frequency:f64,
	pub gamma:f64,
}

impl Component for AtomInfo{
	type Storage = VecStorage<Self>;	
}

pub struct NewlyCreated;

impl Component for NewlyCreated{
	type Storage = VecStorage<Self>;
}