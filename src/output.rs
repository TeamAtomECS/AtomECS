extern crate specs;
use crate::maths::Maths;
use crate::constant;
use crate::atom::{Position,Velocity,Force,Interaction_lasers,rand_kick};
use crate::initiate::{timestep,step};

use specs::{System,Write,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect,Component,VecStorage,Entities,LazyUpdate};

pub struct Print_output;

impl <'a>System <'a> for Print_output{

	// print the output (whatever you want) to the console
	type SystemData = (
								ReadStorage<'a,Interaction_lasers>,
								ReadStorage<'a,Position>,
								ReadStorage<'a,Velocity>,
								ReadStorage<'a,Force>,
								ReadStorage<'a,rand_kick>,
								ReadExpect<'a,step>,
								ReadExpect<'a,timestep>,
								);
	fn run(&mut self, (_lasers,_pos,_vel,_force,_kick,_step,_t):Self::SystemData){
		let time = _t.t * _step.n as f64;
		for (_lasers,_vel,_pos,_force,_kick) in (&_lasers,&_vel,&_pos,&_force,&_kick).join(){
			if _step.n % 100 == 0{
				for inter in &_lasers.content{
					println!("index{},detuning{},force{:?}",inter.index,inter.detuning_doppler,inter.force);
				}
				println!("time{}position{:?},velocity{:?},acc{:?},kick{:?}",time,_pos.pos,_vel.vel,Maths::array_multiply(&_force.force,1./constant::MRb),Maths::array_multiply(&_kick.force,1./constant::MRb));
			}
		//println!("position{:?},velocity{:?}",_pos.pos,_vel.vel);
		}
	}
}
pub struct Atom_output{
	pub number_of_atom : u64,
	pub total_velcotiy:[f64;3],
}

pub struct Detector{

	// a detector with centre at centre and have a dimension of 2*range
	pub centre:[f64;3],
	pub range:[f64;3],
}

impl Component for Detector{
	type Storage = VecStorage<Self>;
}

pub struct Detecting_atom;

impl <'a>System<'a> for Detecting_atom{
	type SystemData = (
								Entities<'a>,
								ReadStorage<'a,Detector>,
								WriteStorage<'a,Position>,
								WriteStorage<'a,Velocity>,
								WriteExpect<'a,Atom_output>,
								Read<'a,LazyUpdate>,
								);
	fn run(&mut self, (mut ent,detector,mut _pos,mut _vel,mut atom_output,lazy):Self::SystemData){
		//check if an atom is within the detector
		for (detector) in (&detector).join(){
		for (ent,mut _vel,_pos) in (&ent,&mut _vel,&_pos).join(){
			if if_detect(&detector,&_pos.pos){
				atom_output.number_of_atom = atom_output.number_of_atom + 1;
				atom_output.total_velcotiy = Maths::array_addition(&atom_output.total_velcotiy,&_vel.vel);
				lazy.remove::<Position>(ent);
				lazy.remove::<Velocity>(ent);
			}
			// what to do with the detected data
		}
		}
	}
}
// a function here just for convenience
	pub fn if_detect (_detector:&Detector, position:&[f64;3]) -> bool{
		let mut result = true;
		for i in (0..3){
			result = result && (position[i]<(_detector.centre[i]+_detector.range[i]))&&(position[i]>(_detector.centre[i]-_detector.range[i]));
		}
	result
	}
	#[test]
	fn test_if_detect(){
		assert!(if_detect(&Detector{centre:[0.,0.,0.],range:[1.,1.,1.]},&[0.9,0.8,-0.7]));
	}
	
	
pub struct Print_detect;

impl <'a>System<'a> for Print_detect{
	type SystemData = (WriteExpect<'a,Atom_output>);
	fn run(&mut self, (atom_output):Self::SystemData){
		let average_vel = Maths::array_multiply(&atom_output.total_velcotiy,1./atom_output.number_of_atom as f64);
		println!("atom captured{},average velocity{:?}",atom_output.number_of_atom,average_vel);
	}
}