use crate::maths::Maths;
	extern crate rand;
	use rand::Rng;
use crate::initiate::*;
use crate::constant::pi as pi;
extern crate specs;
use crate::atom::*;
use specs::{System,Write,ReadStorage,WriteStorage,Join,Read,ReadExpect,WriteExpect,Component,VecStorage,Entities,LazyUpdate};


pub fn velocity_generate(_t:f64,_mass:f64,_dir:&[f64;3])->[f64;3]{
let mut rng = rand::thread_rng();
	let v_mag = Maths::maxwell_generate(_t,_mass);
	let dir = Maths::norm(&_dir);
	let dir_1 = Maths::norm(&[1.0,0.0,-dir[0]/dir[2]]);
	let dir_2 = Maths::norm(&[1.0,(dir[1].powf(2.0)-1.0)/dir[0]/dir[1],dir[2]/dir[0]]);
	let mut rng = rand::thread_rng();
	let theta = Maths::jtheta_gen();
	let theta2 = rng.gen_range(0.0, 2.0*pi);
	println!("angle one {},angle two {}",theta,theta2);
	let dir_div = Maths::array_addition(&Maths::array_multiply(&dir_1,theta.sin()*theta2.cos()),&Maths::array_multiply(&dir_2,theta.sin()*theta2.sin()));
	let dirf = Maths::array_addition(&Maths::array_multiply(&dir,theta.cos()),&dir_div);
	println!("{:?}",Maths::array_multiply(&dirf,v_mag));
	Maths::array_multiply(&dirf,v_mag)
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

pub struct Atom_create;

impl <'a> System<'a> for Atom_create{
	type SystemData = (Entities<'a>,
								ReadStorage<'a,Oven>,
								ReadStorage<'a,Atom_info>,
								ReadStorage<'a,Position>,
								ReadStorage<'a,Velocity>,
								Read<'a,LazyUpdate>,
								);
	
	fn run (&mut self, (entities,_oven,atom,_pos,_vel,updater):Self::SystemData){
		
				let mut content = Vec::new();

		let empty_laser = Interaction_lasers{content};

		
		let mut rng = rand::thread_rng();

		for (_oven,atom) in (&_oven,&atom).join(){
		let mass = atom.mass;
		let dir = _oven.direction.clone();
		let size = _oven.size.clone();
		let _iposition = _oven.position.clone();
		for i in 0.._oven.number{
			let new_atom = entities.create();
			let new_vel = velocity_generate(_oven.temperature,atom.mass,&dir);
			let pos1 = rng.gen_range(- 0.5*size[0], 0.5*size[0]);
			let pos2 = rng.gen_range(- 0.5*size[1], 0.5*size[1]);
			let pos3 = rng.gen_range(- 0.5*size[2], 0.5*size[2]);
			let start_position = [_iposition[0]+pos1,_iposition[1]+pos2,_iposition[2]+pos3];
			updater.insert(new_atom,Position{pos:start_position});
			updater.insert(new_atom,Velocity{vel:new_vel});
			updater.insert(new_atom,Force{force:[0.,0.,0.]});
			updater.insert(new_atom,Atom_info{mass:atom.mass,mup:atom.mup,muz:atom.muz,mum:atom.mum,frequency:atom.frequency,gamma:atom.gamma});

			println!("atom created");
		}

		}
		for (ent,atom,vel) in (&entities,&atom,&_vel).join(){

			println!("WTF");
		}
	}
}

pub struct Atom_initiate_MOT;

impl <'a> System<'a> for Atom_initiate_MOT{
	type SystemData = (Entities<'a>,
								ReadStorage<'a,Atom_info>,
								ReadStorage<'a,Position>,
								ReadStorage<'a,Velocity>,
								Read<'a,LazyUpdate>,
								ReadStorage<'a,Laser>,
								);
	
	fn run (&mut self, (ent,atom,position,velocity,updater,_laser):Self::SystemData){
		let mut content = Vec::new();
		println!("atom initiate 1");
		for (_laser) in (&_laser).join(){
			content.push(interaction_laser{	
			index:_laser.index,
			intensity:0.,
			polarization:0.,
			wavenumber:[0.,0.,0.],
			detuning_doppler:0.,
			force:[0.,0.,0.],
			})
		}

		let empty_laser = Interaction_lasers{content};
		for (ent,atom,position,velocity) in (&ent,&atom,&position,&velocity).join(){
			let empty_mag = Mag_sampler{mag_sampler:[0.,0.,0.]};
			updater.insert(ent,rand_kick{force:[0.,0.,0.]});
			updater.insert(ent,empty_mag);
			updater.insert(ent,empty_laser.clone());
			println!("atom initiated");
		}
	}
}