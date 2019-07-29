extern crate specs;
use specs::{System,Write,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect,Component,VecStorage};
use crate::maths::Maths;
use crate::constant;
use crate::constant::hbar as hbar;
pub struct Create_DE;
pub mod atom_create;

pub struct step{
	pub n : u64,
}

pub struct timestep{
	pub t:f64,
}


// when initializing the world, all the environmental variable will be added to the world as resources





pub struct Atom_info{
	pub mass:f64,
	pub mup:f64,
	pub mum:f64,
	pub muz:f64,
	pub frequency:f64,
	pub gamma:f64,
}

impl Component for Atom_info{
	type Storage = VecStorage<Self>;	
}