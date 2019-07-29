mod initiate;
mod maths;
mod constant;
mod atom;
mod update;
mod output;
use crate::constant::pi as pi;
use crate::initiate::step;
use crate::atom::Position;
use crate::atom::{Velocity,Interaction_lasers,Force,Mag_sampler,rand_kick};
use crate::initiate::{timestep,Laser,Mag_field_gaussian,Atom_info};
use crate::maths::Maths;
use crate::update::{Update_sampler,Update_position_euler,Update_force,Update_interaction_laser,Update_laser,Update_rand_kick};
use crate::initiate::atom_create::{Atom_create,Oven,Atom_initiate_MOT};
use specs::{World,Builder,DispatcherBuilder,RunNow};
use output::Print_output;
fn main() {
   // create the world
   let mut exp_MOT = World::new();
   
	// create the resources and component, and entities for experimental setup
	exp_MOT.register::<Velocity>();
	exp_MOT.register::<Position>();
	exp_MOT.register::<Oven>();
	exp_MOT.register::<Force>();
	exp_MOT.register::<Atom_info>();
	exp_MOT.register::<Laser>();
	exp_MOT.register::<Mag_sampler>();
	exp_MOT.register::<Interaction_lasers>();
	exp_MOT.register::<Mag_field_gaussian>();
	exp_MOT.register::<rand_kick>();
	
	//component for the experiment
	let Rb_atom = Atom_info{	mass:constant::MRb,
	mup:constant::mup,
	mum:constant::mum,
	muz:constant::muz,
	frequency:constant::atom_frequency,
	gamma:constant::trans_width
	};
	exp_MOT.add_resource(step{n:0});
	let mag= Mag_field_gaussian{
		gradient:0.002,
		centre:[0.,0.,0.],
	};
	exp_MOT.create_entity().with(mag).build();
	// adding all six lasers
	let laser_1 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,2.0*pi/(461e-9)],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::c/461e-9,
		index:1,
	};
		let laser_2 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,-2.0*pi/(461e-9)],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::c/461e-9,
		
		index:2,
	};
		let laser_3 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,2.0*pi/(461e-9),0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::c/461e-9,
		index:3,
	};
		let laser_4 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,-2.0*pi/(461e-9),0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::c/461e-9,
		index:4,
	};
		let laser_5 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[2.0*pi/(461e-9),0.,0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::c/461e-9,
		index:5,
	};
		let laser_6 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[-2.0*pi/(461e-9),0.,0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::c/461e-9,
		index:6,
	};
	//six laser introduced
	exp_MOT.create_entity().with(laser_1).build();
	exp_MOT.create_entity().with(laser_2).build();
	exp_MOT.create_entity().with(laser_3).build();
	exp_MOT.create_entity().with(laser_4).build();
	exp_MOT.create_entity().with(laser_5).build();
	exp_MOT.create_entity().with(laser_6).build();
	
	
	exp_MOT.add_resource(timestep{t:1e-6});
	// initiate
		// build a oven
	exp_MOT.create_entity().with(Oven{temperature:200.,position:[0.1,0.1,0.1],direction:[1e-6,1e-6,1.],number:1,size:[1e-2,1e-2,1e-2]})
	.with(Rb_atom).build();
		// initiator dispatched
	let mut init_dispatcher=DispatcherBuilder::new()
			.with(Atom_create,"atomcreate",&[])
      	.build();
		
	//init_dispatcher.setup(&mut exp_MOT.res);
	init_dispatcher.dispatch(&mut exp_MOT.res);
	exp_MOT.maintain();
	//two initiators cannot be dispatched at the same time apparently for some unknown reason
	let mut init_dispatcher2=DispatcherBuilder::new().with(Atom_initiate_MOT, "initiate", &[]).build();
	init_dispatcher2.dispatch(&mut exp_MOT.res);
	// run loop
	let mut runner=DispatcherBuilder::new().
	with(Update_laser,"updatelaser",&[]).
	with(Update_sampler,"updatesampler",&[]).
	with(Update_interaction_laser,"updateinter",&["updatelaser","updatesampler"]).
	with(Update_rand_kick,"update_kick",&["updateinter"]).
	with(Update_force,"updateforce",&["update_kick","updateinter"]).
	with(Update_position_euler,"updatepos",&["update_kick"]).
	with(Print_output,"print",&["updatepos"]).build();
	runner.setup(&mut exp_MOT.res);
	for i in 0..2000{
		runner.dispatch(&mut exp_MOT.res);
		exp_MOT.maintain();
		//println!("t{}",time);
	}
	
}
