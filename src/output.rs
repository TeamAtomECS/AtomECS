extern crate specs;
use crate::maths;
use crate::atom::*;
use crate::laser::InteractionLaserALL;
use crate::integrator::{Timestep,Step};
use crate::constant;

use specs::{System,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect,Component,VecStorage,Entities,LazyUpdate,HashMapStorage};

pub struct PrintOutputSytem;

impl <'a>System <'a> for PrintOutputSytem{

	// print the output (whatever you want) to the console
	type SystemData = (
								ReadStorage<'a,InteractionLaserALL>,
								ReadStorage<'a,Position>,
								ReadStorage<'a,Velocity>,
								ReadStorage<'a,Atom>,
								ReadStorage<'a,Force>,
								ReadStorage<'a,RandKick>,
								ReadExpect<'a,Step>,
								ReadExpect<'a,Timestep>,
								);
	fn run(&mut self, (_lasers,_pos,_vel,_,_force,_kick,_step,_t):Self::SystemData){
		let _time = _t.t * _step.n as f64;
		for (_lasers,_vel,_pos,_force,_kick) in (&_lasers,&_vel,&_pos,&_force,&_kick).join(){
			if _step.n % 100 == 0{
				for _inter in &_lasers.content{
					//println!("index{},detuning{},force{:?}",inter.index,inter.detuning_doppler,inter.force);
				}
				println!("time{}position{:?},velocity{:?},acc{:?},kick{:?}",_time,_pos.pos,_vel.vel,maths::array_multiply(&_force.force,1./constant::AMU/87.),maths::array_multiply(&_kick.force,1./constant::AMU/87.));
			}
		//println!("position{:?},velocity{:?}",_pos.pos,_vel.vel);
		}
	}
}
pub struct AtomOuput{
	pub number_of_atom : u64,
	pub total_velocity:[f64;3],
}

pub struct Detector{

	// a detector with centre at centre and have a dimension of 2*range
	pub centre:[f64;3],
	pub range:[f64;3],
}

impl Component for Detector{
	type Storage = HashMapStorage<Self>;
}

pub struct DetectingAtomSystem;

impl <'a>System<'a> for DetectingAtomSystem{
	type SystemData = (
								Entities<'a>,
								ReadStorage<'a,RingDetector>,
								ReadStorage<'a,Detector>,
								WriteStorage<'a,Position>,
								WriteStorage<'a,Velocity>,
								WriteExpect<'a,AtomOuput>,
								Read<'a,LazyUpdate>,
								);
	fn run(&mut self, (ent,ring_detector, detector,mut _pos,mut _vel,mut atom_output,lazy):Self::SystemData){
		//check if an atom is within the detector
		for detector in (&detector).join(){
		for (ent,mut _vel,_pos) in (&ent,&mut _vel,&_pos).join(){
			if if_detect(&detector,&_pos.pos){
				atom_output.number_of_atom = atom_output.number_of_atom + 1;
				println!("detected velocity{:?},position{:?}",_vel.vel,_pos.pos);
				atom_output.total_velocity = maths::array_addition(&atom_output.total_velocity,&_vel.vel);
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
		for i in 0..3{
			result = result && (position[i]<(_detector.centre[i]+_detector.range[i]))&&(position[i]>(_detector.centre[i]-_detector.range[i]));
		}
	result
	}
	#[test]
	fn test_if_detect(){
		assert!(if_detect(&Detector{centre:[0.,0.,0.],range:[1.,1.,1.]},&[0.9,0.8,-0.7]));
	}
pub struct RingDetector{
	/// a detector with the shape of a ring
	/// could be used in the "reversed" simulation
	/// could also be used as a type of boudary in the experiment
	pub centre:[f64;3],
	pub radius:f64,
	pub width:f64,
	pub thickness:f64,
};

impl Component for RingDetector{
	type Storage = HashMapStorage<Self>;
}


pub struct PrintDetectSystem;

impl <'a>System<'a> for PrintDetectSystem{
	//print the final output of a detector
	type SystemData = (WriteExpect<'a,AtomOuput>);
	fn run(&mut self, atom_output:Self::SystemData){
		let average_vel = maths::array_multiply(&atom_output.total_velocity,1./atom_output.number_of_atom as f64);
		println!("atom captured{},average velocity{:?}",atom_output.number_of_atom,average_vel);
	}
}

pub struct FileOutputSystem;
