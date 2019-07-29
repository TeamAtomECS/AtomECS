extern crate specs;
use specs::{Component,VecStorage,System,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect};
use crate::atom::*;
use crate::magnetic::*;
use crate::initiate::Atom_info;
use crate::constant::hbar as hbar;
use crate::maths::Maths;
use crate::constant;

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
pub struct Update_interaction_laser;
impl <'a> System<'a> for Update_interaction_laser{
	type SystemData = (
									ReadStorage<'a,Position>,
									ReadStorage<'a,Velocity>,
									ReadStorage<'a,Mag_sampler>,
									WriteStorage<'a,Interaction_lasers>,
									ReadStorage<'a,Atom_info>,
									);
		
	fn run(&mut self,(_pos,_vel,_mag,mut _inter,_atom):Self::SystemData){
		
		for (_vel,_pos,_mag,mut _inter,_atom) in (&_vel,&_pos,&_mag,&mut _inter,&_atom).join(){
			//println!("laser interaction updated");
			let mag_field = _mag.mag_sampler;
			let Br = Maths::modulus(&mag_field);
			for inter in &mut _inter.content{
				let _mup = _atom.mup;
				let _mum = _atom.mum;
				let _muz = _atom.muz;	
				let s0 = inter.intensity/constant::sat_inten;
				let omega = Maths::modulus(&inter.wavenumber) * constant::c;
				let wave_vector = inter.wavenumber;
				let p = inter.polarization;
				let gamma = _atom.gamma;
				let atom_frequency = _atom.frequency;
				let costheta = Maths::dot_product(&wave_vector,&mag_field) / Maths::modulus(&wave_vector)/Maths::modulus(&mag_field);
				let detuning = omega - atom_frequency * 2.0* constant::pi - Maths::dot_product(&wave_vector,&_vel.vel);
				
				let scatter1 = (0.25*(p*costheta + 1.).powf(2.)*gamma/2./(1.+s0+4.*(detuning - _mup/hbar*Br).powf(2.)/gamma.powf(2.)));
				let scatter2 = (0.25*(p*costheta - 1.).powf(2.)*gamma/2./(1.+s0+4.*(detuning - _mum/hbar*Br).powf(2.)/gamma.powf(2.)));
				let scatter3 = 0.5*(1. - costheta.powf(2.))*gamma/2./(1.+s0+4.*(detuning - _muz/hbar*Br).powf(2.)/gamma.powf(2.));
				let force_new = Maths::array_multiply(&wave_vector,s0*hbar*(scatter1+scatter2+scatter3));
				
				inter.force =force_new;
				inter.detuning_doppler=detuning;
			}
		}
	}
}

pub struct Update_laser;

impl <'a> System<'a> for Update_laser{
	type SystemData = ( ReadStorage<'a,Position>,
									ReadStorage<'a,Laser>,
									WriteStorage<'a,Interaction_lasers>
									);
		
	fn run(&mut self,(_pos,_laser,mut _inter):Self::SystemData){
		
		//update the sampler for laser, namely intensity, wavenumber? , polarization
		for (mut _inter,_pos) in (&mut _inter,&_pos).join(){
			//println!("laser updated");
			for inter in &mut _inter.content{
			for (_laser) in (&_laser).join(){
				if _laser.index == inter.index{
					let rela_cood = Maths::array_addition(&_pos.pos,&Maths::array_multiply(&_laser.centre,-1.));
					let distance = Maths::modulus(&Maths::cross_product(&_laser.wavenumber,&rela_cood))/Maths::modulus(&_laser.wavenumber);
					let laser_inten = _laser.power*Maths::gaussian_dis(_laser.std,distance);
					inter.intensity = laser_inten;
					inter.wavenumber = _laser.wavenumber;
					inter.polarization = _laser.polarization;
				}
			}
			}
		}
	}
}
