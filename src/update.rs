extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect};

use crate::atom::*;
use crate::laser::Interaction_lasers;
use crate::maths::Maths;
use crate::initiate::*;
use crate::constant::hbar as hbar;
use crate::constant;
extern crate rand;
use rand::Rng;


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
