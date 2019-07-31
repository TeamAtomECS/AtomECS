use crate::constant as constant;
use crate::constant::PI as PI;
use crate::integrator::{Timestep,Step};
use crate::atom::{Atom,Mass,Position,Velocity,Force,RandKick};
use crate::initiate::{AtomInfo,NewlyCreated};
use crate::laser::*;
use crate::magnetic::*;
use crate::initiate::atom_create::{OvenCreateAtomsSystem,Oven};
use crate::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,RunNow};
use crate::output::{PrintOutputSytem,Detector,DetectingAtomSystem,PrintDetectSystem,AtomOuput};
use crate::initiate::ecs;
#[allow(dead_code)]
pub fn create(){
   // create the world
   let mut exp_mot = World::new();
	// create the resources and component, and entities for experimental setup
	ecs::register_lazy(&mut exp_mot);
	//component for the experiment
	let rb_atom = AtomInfo{
	mass:87,
	mup:constant::MUP,
	mum:constant::MUM,
	muz:constant::MUZ,
	frequency:constant::ATOMFREQUENCY,
	gamma:constant::TRANSWIDTH,
	saturation_intensity: constant::SATINTEN
	};
	exp_mot.add_resource(Step{n:0});
	exp_mot.add_resource(AtomOuput{number_of_atom:0,total_velocity:[0.,0.,0.]});
	let mag= QuadrupoleField3D{
		gradient:0.002
	};
	exp_mot.create_entity().with(mag).with(Position{pos:[0.,0.,0.]}).build();
	// adding all six lasers
	let laser_1 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,2.0*PI/(461e-9)],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:1,
	};
		let laser_2 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,-2.0*PI/(461e-9)],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		
		index:2,
	};
		let laser_3 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,2.0*PI/(461e-9),0.],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:3,
	};
		let laser_4 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,-2.0*PI/(461e-9),0.],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:4,
	};
		let laser_5 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[2.0*PI/(461e-9),0.,0.],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:5,
	};

	//six laser introduced
	exp_mot.create_entity().with(laser_1).build();
	exp_mot.create_entity().with(laser_2).build();
	exp_mot.create_entity().with(laser_3).build();
	exp_mot.create_entity().with(laser_4).build();
	exp_mot.create_entity().with(laser_5).build();
	//detector introduced
	
	exp_mot.create_entity().with(Detector{centre:[0.2,0.,0.],range:[0.05,0.1,0.1]}).build();
	
	exp_mot.add_resource(Timestep{delta:1e-6});
	// initiate
		// build a oven
	exp_mot.create_entity()
	.with(Oven{temperature:200.,direction:[1e-6,1e-6,1.],number:1,size:[1e-2,1e-2,1e-2]})
	.with(rb_atom)
	.with(Mass{value:87.})
	.with(Position{pos:[0.0,0.0,0.0]})
	.build();
		// initiator dispatched
	let mut init_dispatcher=DispatcherBuilder::new()
			.with(OvenCreateAtomsSystem,"atomcreate",&[])
      	.build();
		
	//init_dispatcher.setup(&mut exp_MOT.res);
	init_dispatcher.dispatch(&mut exp_mot.res);
	exp_mot.maintain();
	//two initiators cannot be dispatched at the same time apparently for some unknown reason
	let mut init_dispatcher2=DispatcherBuilder::new().with(AttachLaserComponentsToNewlyCreatedAtomsSystem, "initiate", &[]).build();
	init_dispatcher2.dispatch(&mut exp_mot.res);

	let mut runner = ecs::create_simulation_dispatcher();

	runner.setup(&mut exp_mot.res);
	for _i in 0..10000{
		runner.dispatch(&mut exp_mot.res);
		exp_mot.maintain();
	}
	let mut print_detect = PrintDetectSystem;
	print_detect.run_now(&exp_mot.res);	
}