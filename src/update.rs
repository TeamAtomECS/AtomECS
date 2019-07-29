extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,ReadExpect,WriteExpect};

use crate::atom::*;
use crate::laser::InteractionLaserALL;
use crate::maths::Maths;
use crate::initiate::*;
use crate::constant;
extern crate rand;
use rand::Rng;


pub struct UpdateEuler;


// update function will update the the position and the velocity of the particle in a given timespan/ timestep
impl <'a> System<'a> for UpdateEuler{
	type SystemData = (	WriteStorage<'a,Position>,
									WriteStorage<'a,Velocity>,
									ReadExpect<'a,Timestep>,
									WriteExpect<'a,Step>,
									ReadStorage<'a,Force>,
									ReadStorage<'a,AtomInfo>
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


pub struct UpdateForce;

impl <'a>System<'a> for UpdateForce{
	// this system will update the force component for atoms based on interaction with lasers and random kick
	type SystemData = ( WriteStorage<'a,Force>,
									ReadStorage<'a,InteractionLaserALL>,
									ReadStorage<'a,RandKick>
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

pub struct UpdateRandKick;
//this system must be ran after update_force
impl <'a>System<'a> for UpdateRandKick{
	type SystemData = (ReadStorage<'a,InteractionLaserALL>,
								WriteStorage<'a,RandKick>,
								ReadExpect<'a,Timestep>,
								ReadStorage<'a,AtomInfo>);	
	fn run(&mut self, (_inter,mut _kick,_t,_atom):Self::SystemData){
		// to the best of the knowledge, the number of actual random kick should be calculated using a possoin distribution
		for (_inter,mut _kick,_atom) in (&_inter,&mut _kick,&_atom).join(){
			//this system will look at forces due to interaction with all the lasers and calculate the corresponding number of random kick involved
			let mut total_impulse = 0.0 ; 
			_kick.force =[0.,0.,0.];
			for interaction in &_inter.content{
				total_impulse = total_impulse + Maths::modulus(&interaction.force)*_t.t;
			}
			let momentum_photon = constant::HBAR * 2.*constant::PI*_atom.frequency/constant::c;
			let mut num_kick = total_impulse/ momentum_photon;
			//num_kick will be the expected number of random kick involved
			loop{
				if num_kick >1.{
					// if the number is bigger than 1, a random kick will be added with direction random
					num_kick = num_kick - 1.;
					_kick.force = Maths::array_addition(&_kick.force,&Maths::array_multiply(&Maths::random_direction(),momentum_photon/_t.t));
				}
				else{
					// if the remaining kick is smaller than 0, there is a chance that the kick is random
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