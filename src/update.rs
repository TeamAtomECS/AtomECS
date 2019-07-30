extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,RunNow,World,ReadExpect,Builder};

use crate::atom::*;
use crate::laser::InteractionLaserALL;
use crate::maths;
use crate::integrator::*;
use crate::initiate::*;
use crate::constant;
extern crate rand;
use rand::Rng;

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
				new_force = maths::array_addition(&new_force,&_inter.force);
			}
			_force.force = new_force;
		}
		for (mut _force,_kick) in (&mut _force,&kick).join(){
			_force.force = maths::array_addition(&_kick.force,&_force.force);
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
				total_impulse = total_impulse + maths::modulus(&interaction.force)*_t.t;
			}
			let momentum_photon = constant::HBAR * 2.*constant::PI*_atom.frequency/constant::C;
			let mut num_kick = total_impulse/ momentum_photon;
			//num_kick will be the expected number of random kick involved
			loop{
				if num_kick >1.{
					// if the number is bigger than 1, a random kick will be added with direction random
					num_kick = num_kick - 1.;
					_kick.force = maths::array_addition(&_kick.force,&maths::array_multiply(&maths::random_direction(),momentum_photon/_t.t));
				}
				else{
					// if the remaining kick is smaller than 0, there is a chance that the kick is random
					let mut rng = rand::thread_rng();
					let result = rng.gen_range(0.0, 1.0);
					if result < num_kick{
						_kick.force = maths::array_addition(&_kick.force,&maths::array_multiply(&maths::random_direction(),momentum_photon/_t.t));
					}
					break;
				}
			}
		}
	}
}

#[cfg(test)]
pub mod tests {

	use super::*;
	extern crate specs;
	use crate::laser::InteractionLaser;

	/// Tests the correct implementation of update force
	#[test]
	
	/// Tests the correct implementation of the magnetics systems and dispatcher.
	/// This is done by setting up a test world and ensuring that the magnetic systems perform the correct operations on test entities.
	#[test]
	fn test_magnetics_systems()
	{
		let mut test_world = World::new();
		test_world.register::<RandKick>();
		test_world.register::<Force>();
		test_world.register::<InteractionLaserALL>();

		let mut content = Vec::new();
		content.push(InteractionLaser{wavenumber:[1.,1.,2.],index:1,intensity:1.,polarization:1.,detuning_doppler:1.,force:[1.,0.,0.]});
		content.push(InteractionLaser{wavenumber:[1.,1.,2.],index:2,intensity:1.,polarization:1.,detuning_doppler:1.,force:[2.,0.,0.]});

		let test_interaction = InteractionLaserALL{content};
		let test_kick = RandKick{force:[1.,0.,0.]};
		let sample_entity= test_world.create_entity().
		with(test_interaction).
		with(test_kick).
		with(Force{force:[0.,0.,0.]}).build();

		let mut update_test = UpdateForce;
		update_test.run_now(&test_world.res);



		let samplers = test_world.read_storage::<Force>();
		let sampler = samplers.get(sample_entity);
		assert_eq!(sampler.expect("entity not found").force,[4.,0.,0.]);
	}
}	