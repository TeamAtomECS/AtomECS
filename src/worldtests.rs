use crate::constant as constant;
use crate::constant::PI as PI;
use crate::integrator::{Timestep,Step};
use crate::atom::{Mass,Position,Velocity,Force,RandKick};
use crate::initiate::AtomInfo;
use crate::update::*;
use crate::laser::*;
use crate::magnetic::*;
use crate::initiate::atom_create::{AtomCreate,Oven,AtomInitiateMot};
use crate::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,RunNow};
use crate::output::{PrintOutput,Detector,DetectingAtom,PrintDetect,AtomOuput};
// maybe this will work as an alternative way of testing
#[cfg(test)]
mod tests {

    #[test]
pub fn testing_ground(){
   // create the world
   let mut exp_mot = World::new();
   
	// create the resources and component, and entities for experimental setup
	exp_mot.register::<Velocity>();
	exp_mot.register::<Position>();
	exp_mot.register::<Oven>();
	exp_mot.register::<Force>();
	exp_mot.register::<AtomInfo>();
	exp_mot.register::<Mass>();
	exp_mot.register::<Laser>();
	exp_mot.register::<MagneticFieldSampler>();
	exp_mot.register::<InteractionLaserALL>();
	exp_mot.register::<QuadrupoleField3D>();
	exp_mot.register::<RandKick>();
	
	//component for the experiment
	let rb_atom = AtomInfo{
	mup:constant::MUP,
	mum:constant::MUM,
	muz:constant::MUZ,
	frequency:constant::ATOMFREQUENCY,
	gamma:constant::TRANSWIDTH
	};
	exp_mot.add_resource(Step{n:0});
	let mag= QuadrupoleField3D{gradient:0.002};
	exp_mot.create_entity().with(mag).build();

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

	//six laser introduced
	exp_mot.create_entity().with(laser_1).build();
	exp_mot.create_entity().with(laser_2).build();

	
	
	exp_mot.add_resource(Timestep{t:1e-6});
	// initiate
		// build a oven
	exp_mot.create_entity().with(Oven{temperature:0.,position:[0.1,0.1,0.1],direction:[1e-6,1e-6,1.],number:1,size:[1e-2,1e-2,1e-2]})
	.with(rb_atom).build();
	assert
		// initiator dispatched
	let mut init_dispatcher=DispatcherBuilder::new()
			.with(AtomCreate,"atomcreate",&[])
      	.build();
		
	//init_dispatcher.setup(&mut exp_MOT.res);
	init_dispatcher.dispatch(&mut exp_mot.res);
	exp_mot.maintain();
	//two initiators cannot be dispatched at the same time apparently for some unknown reason
	let mut init_dispatcher2=DispatcherBuilder::new().with(AtomInitiateMot, "initiate", &[]).build();
	init_dispatcher2.dispatch(&mut exp_mot.res);
	// run loop
	let mut runner=DispatcherBuilder::new().
	with(UpdateLaser,"updatelaser",&[]).
	with(Sample3DQuadrupoleFieldSystem,"updatesampler",&[]).
	with(UpdateInteractionLaser,"updateinter",&["updatelaser","updatesampler"]).
	with(UpdateRandKick,"update_kick",&["updateinter"]).
	with(UpdateForce,"updateforce",&["update_kick","updateinter"]).
	with(EulerIntegrationSystem,"updatepos",&["update_kick"]).
	with(PrintOutput,"print",&["updatepos"]).build();
	runner.setup(&mut exp_mot.res);
	for _i in 0..2000{
		runner.dispatch(&mut exp_mot.res);
		exp_mot.maintain();
		//println!("t{}",time);
	}
	
}
}