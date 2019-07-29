extern crate specs;
use specs::{Component, VecStorage};

//this file is simply straightforward
pub struct Position{
	pub pos:[f64;3]
}

pub struct Mass{
	pub mass:f64,
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


pub struct Mag_sampler{
	pub mag_sampler:[f64;3]
}

impl Component for Mag_sampler{
	type Storage = VecStorage<Self>;
}

pub struct interaction_laser{	
	pub index:u64,
	pub intensity:f64,
	pub polarization:f64,
	pub wavenumber:[f64;3],
	pub detuning_doppler:f64,
	pub force:[f64;3],
}

impl interaction_laser{
	
	pub fn clone(&self)-> interaction_laser{
		interaction_laser{index:self.index,intensity:self.intensity,polarization:self.polarization,wavenumber:self.wavenumber.clone(),detuning_doppler:self.detuning_doppler,force:self.force.clone()}
	}
	
}

pub struct Interaction_lasers{
	pub content:Vec<interaction_laser>,
}

impl Component for Interaction_lasers{
	type Storage = VecStorage<Self>;
}

impl Interaction_lasers{
	pub fn clone(&self) -> Interaction_lasers{
		let mut new = Vec::new();
		for i in self.content.iter(){
			new.push(i.clone());
		}
		Interaction_lasers{content:new}
	}
}

pub struct rand_kick{
	pub force:[f64;3]
}

impl Component for rand_kick{
	type Storage = VecStorage<Self>;
}
