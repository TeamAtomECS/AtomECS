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

pub struct DE{
	pub content:Box<Fn(&[f64;3],&[f64;3])->[f64;3]+Send+Sync>,
}
// DE.content will give the closure that relate acceleration with phase spaces

// when initializing the world, all the environmental variable will be added to the world as resources
pub struct Mag_field_gaussian{
	pub gradient:f64,
	pub centre:[f64;3],
}

impl Component for Mag_field_gaussian{
	type Storage = VecStorage<Self>;
}

pub struct Laser_beams{
	pub content:Vec<Laser>,
}

pub struct Laser{
	pub centre:[f64;3],
	pub wavenumber:[f64;3],
	pub polarization:f64,
	pub power:f64,
	pub std:f64,
	pub frequency:f64,
	pub index:u64,
}

impl Component for Laser{
	type Storage = VecStorage<Self>;
}



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