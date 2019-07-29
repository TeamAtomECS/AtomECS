use crate::maths;
extern crate rand;
use rand::Rng;
use crate::initiate::*;
use crate::constant::PI as PI;
extern crate specs;
use crate::atom::*;
use crate::laser::*;
use crate::magnetic::MagneticFieldSampler;
use specs::{System,ReadStorage,Join,Read,Component,VecStorage,Entities,LazyUpdate};


pub fn velocity_generate(_t:f64,_mass:f64,_dir:&[f64;3])->[f64;3]{
	let v_mag = maths::maxwell_generate(_t,_mass);
	let dir = maths::norm(&_dir);
	let dir_1 = maths::norm(&[1.0,0.0,-dir[0]/dir[2]]);
	let dir_2 = maths::norm(&[1.0,(dir[1].powf(2.0)-1.0)/dir[0]/dir[1],dir[2]/dir[0]]);
	let mut rng = rand::thread_rng();
	let theta = maths::jtheta_gen();
	let theta2 = rng.gen_range(0.0, 2.0*PI);
	println!("angle one {},angle two {}",theta,theta2);
	let dir_div = maths::array_addition(&maths::array_multiply(&dir_1,theta.sin()*theta2.cos()),&maths::array_multiply(&dir_2,theta.sin()*theta2.sin()));
	let dirf = maths::array_addition(&maths::array_multiply(&dir,theta.cos()),&dir_div);
	println!("{:?}",maths::array_multiply(&dirf,v_mag));
	maths::array_multiply(&dirf,v_mag)
	//[0.,0.,100.]
}

pub struct Oven{
	pub temperature: f64,
	pub position:[f64;3],
	pub size:[f64;3],
	pub direction:[f64;3],
	pub number:u64,
}

impl Component for Oven{
	type Storage = VecStorage<Self>;
}

pub struct AtomCreate;

impl <'a> System<'a> for AtomCreate{
	type SystemData = (Entities<'a>,
								ReadStorage<'a,Oven>,
								ReadStorage<'a,AtomInfo>,
								ReadStorage<'a,Position>,
								ReadStorage<'a,Velocity>,
								Read<'a,LazyUpdate>,
								);
	
	fn run (&mut self, (entities,_oven,atom,_pos,_vel,updater):Self::SystemData){


	
		
		let mut rng = rand::thread_rng();

		for (_oven,atom) in (&_oven,&atom).join(){
		let dir = _oven.direction.clone();
		let size = _oven.size.clone();
		let _iposition = _oven.position.clone();
		for _i in 0.._oven.number{
			let new_atom = entities.create();
			let new_vel = velocity_generate(_oven.temperature,atom.mass,&dir);
			let pos1 = rng.gen_range(- 0.5*size[0], 0.5*size[0]);
			let pos2 = rng.gen_range(- 0.5*size[1], 0.5*size[1]);
			let pos3 = rng.gen_range(- 0.5*size[2], 0.5*size[2]);
			let start_position = [_iposition[0]+pos1,_iposition[1]+pos2,_iposition[2]+pos3];
			updater.insert(new_atom,Position{pos:start_position});
			updater.insert(new_atom,Velocity{vel:new_vel});
			updater.insert(new_atom,Force{force:[0.,0.,0.]});
			updater.insert(new_atom,AtomInfo{mass:atom.mass,mup:atom.mup,muz:atom.muz,mum:atom.mum,frequency:atom.frequency,gamma:atom.gamma});

			println!("atom created");
		}

		}

	}
}

pub struct AtomInitiateMot;

impl <'a> System<'a> for AtomInitiateMot{
	type SystemData = (Entities<'a>,
								ReadStorage<'a,AtomInfo>,
								ReadStorage<'a,Position>,
								ReadStorage<'a,Velocity>,
								Read<'a,LazyUpdate>,
								ReadStorage<'a,Laser>,
								);
	
	fn run (&mut self, (ent,atom,position,velocity,updater,_laser):Self::SystemData){
		let mut content = Vec::new();
		println!("atom initiate 1");
		for _laser in (&_laser).join(){
			content.push(InteractionLaser{	
			index:_laser.index,
			intensity:0.,
			polarization:0.,
			wavenumber:[0.,0.,0.],
			detuning_doppler:0.,
			force:[0.,0.,0.],
			})
		}

		let empty_laser = InteractionLaserALL{content};
		for (ent,_atom,_position,_velocity) in (&ent,&atom,&position,&velocity).join(){
			let empty_mag = MagneticFieldSampler{field:[0.,0.,0.], magnitude:0.};
			updater.insert(ent,RandKick{force:[0.,0.,0.]});
			updater.insert(ent,empty_mag);
			updater.insert(ent,empty_laser.clone());
			println!("atom initiated");
		}
	}
}