extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect};
use crate::initiate::DE;
use crate::atom::*;
use crate::maths::Maths;
use crate::initiate::*;
use crate::constant::hbar as hbar;
use crate::constant;
extern crate rand;
use rand::Rng;

pub struct Update_sampler;

impl <'a> System<'a> for Update_sampler{
		type SystemData = (WriteStorage<'a,Mag_sampler>,
									ReadStorage<'a,Position>,
									ReadStorage<'a,Mag_field_gaussian>,
									);
	fn run(&mut self,(mut _sampler,pos,_mag_gauss):Self::SystemData){
		
		for (_mag_gauss) in (&_mag_gauss).join(){
			
			for (pos,mut sampler) in (&pos,&mut _sampler).join(){
			//println!("sampler updated");
				let _gradient = _mag_gauss.gradient;
				let _centre = _mag_gauss.centre;
				let rela_pos = Maths::array_addition(&pos.pos,&Maths::array_multiply(&_centre,-1.));
				sampler.mag_sampler = Maths::array_multiply(&[-rela_pos[0],-rela_pos[1],2.0*rela_pos[2]],_gradient);
			}
		}
		//println!("position{:?},velocity{:?}",_pos.pos,_vel.vel);
	}
}

pub struct Update_position_euler;


// update function will update the the position and the velocity of the particle in a given timespan/ timestep
impl <'a> System<'a> for Update_position_euler{
	type SystemData = (	WriteStorage<'a,Position>,
									WriteStorage<'a,Velocity>,
									ReadExpect<'a,timestep>,
									WriteExpect<'a,step>,
									ReadStorage<'a,Force>,
									ReadStorage<'a,Atom_info>
									);
		
	fn run(&mut self,(mut _pos,mut _vel,_t,mut _step,_force,_atom):Self::SystemData){
		
		_step.n = _step.n +1;
		for (mut _vel,mut _pos,_force,_atom) in (&mut _vel,&mut _pos,&_force,&_atom).join(){
			//println!("euler method used");
			let _mass = _atom.mass;
			_vel.vel = Maths::array_addition(&_vel.vel,&Maths::array_multiply(&_force.force,1./_mass*_t.t));
			_pos.pos = Maths::array_addition(&_pos.pos,&Maths::array_multiply(&_vel.vel,_t.t));

		}
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

pub struct Update_force;

impl <'a>System<'a> for Update_force{
	type SystemData = ( WriteStorage<'a,Force>,
									ReadStorage<'a,Interaction_lasers>,
									ReadStorage<'a,rand_kick>
									);
	fn run(&mut self,(mut _force,inter,kick):Self::SystemData){
		for (mut _force, inter) in (&mut _force,&inter).join(){
			let mut new_force = [0.,0.,0.];
			//println!("force updated");
		
			for _inter in inter.content.iter(){
				new_force = Maths::array_addition(&new_force,&_inter.force);
			}
			_force.force = new_force;
		}
		for (mut _force,_kick) in (&mut _force,&kick).join(){
			_force.force = Maths::array_addition(&_kick.force,&_force.force);
		}
	}
}

pub struct Update_rand_kick;
//this system must be ran after update_force
impl <'a>System<'a> for Update_rand_kick{
	type SystemData = (ReadStorage<'a,Interaction_lasers>,
								WriteStorage<'a,rand_kick>,
								ReadExpect<'a,timestep>,
								ReadStorage<'a,Atom_info>);	
	fn run(&mut self, (_inter,mut _kick,_t,_atom):Self::SystemData){
		for (_inter,mut _kick,_atom) in (&_inter,&mut _kick,&_atom).join(){
			let mut total_impulse = 0.0 ; 
			_kick.force =[0.,0.,0.];
			for interaction in &_inter.content{
				total_impulse = total_impulse + Maths::modulus(&interaction.force)*_t.t;
			}
			let momentum_photon = constant::hbar * 2.*constant::pi*_atom.frequency/constant::c;
			let mut num_kick = total_impulse/ momentum_photon;
			loop{
				if num_kick >1.{
					num_kick = num_kick - 1.;
					_kick.force = Maths::array_addition(&_kick.force,&Maths::array_multiply(&Maths::random_direction(),momentum_photon/_t.t));
				}
				else{
					
					let mut rng = rand::thread_rng();
					let result = rng.gen_range(0.0, 1.0);
					if result < num_kick{
						_kick.force = Maths::array_addition(&_kick.force,&Maths::array_multiply(&Maths::random_direction(),momentum_photon/_t.t));
					}
					break;
				}
			}
		}
	}
}


pub struct collision;
