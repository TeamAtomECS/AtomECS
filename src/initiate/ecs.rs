use crate::atom::{Atom,Mass,Position,Velocity,Force,RandKick,Gravity};
use crate::initiate::{AtomInfo,NewlyCreated,DeflagNewAtomsSystem};
use crate::integrator::{Timestep,Step};
use crate::update::*;
use crate::laser::*;
use crate::magnetic::*;
use crate::initiate::atom_create::{Oven};
use crate::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,Dispatcher};
use crate::output::*;
/// register some general component
pub fn register_component_general(world: &mut World) {

		world.register::<Position>();
		world.register::<Velocity>();
		world.register::<Force>();
		world.register::<Mass>();
}

/// register component related to atom creation
pub fn register_component_atomcreation(world: &mut World) {

		world.register::<Oven>();
		world.register::<AtomInfo>();
        world.register::<Atom>();
		world.register::<NewlyCreated>();
}

/// register component related to forces other than laser force
pub fn register_component_otherforce(world: &mut World) {
		world.register::<Gravity>();
		world.register::<RandKick>();
}

/// register component for output system
pub fn register_component_output(world: &mut World) {
		world.register::<Detector>();
		world.register::<RingDetector>();
}

/// if you are lazy and have no idea what you want, use this function to register everything
pub fn register_lazy(mut world: &mut World){
    register_component_output(&mut world);
    register_component_otherforce(&mut world);
    register_component_atomcreation(&mut world);
    register_component_general(&mut world);
    register_component_magnetic(&mut world);
    register_component_laser(&mut world);

}

///  add general update system to dispatcher 
fn add_systems_to_dispatch_general_update(builder: DispatcherBuilder<'static,'static>, deps: &[&str]) -> DispatcherBuilder<'static,'static> {
	builder.
	with(UpdateRandKick,"update_kick",deps).
	with(UpdateForce,"updateforce",&["update_kick"]).
    with(DeflagNewAtomsSystem,"deflag",&["updateforce"]).
	with(EulerIntegrationSystem,"updatepos",&["update_kick"]).
	with(PrintOutputSytem,"print",&["updatepos"]).
	with(DetectingAtomSystem,"detect",&["updatepos"])
}

/// create the running dispatcher
pub fn create_dispatcher_running()->Dispatcher<'static,'static>{
    let builder=DispatcherBuilder::new();
    let builder_1 = add_systems_to_dispatch_magnetic(builder, &[]);
    let builder_2 = add_systems_to_dispatch_laser(builder_1, &["magnetics_magnitude"]);
    let builder_3 = add_systems_to_dispatch_general_update(builder_2, &["updatelaserinter"]);
    let dispatcher = builder_3.build();
    dispatcher
}

/// add resources that is necessary easily
pub fn register_resources_lazy(mut world: &mut World){
    world.add_resource(Timestep{delta:1e-6});
    world.add_resource(Step{n:0});
	world.add_resource(AtomOuput{number_of_atom:0,total_velocity:[0.,0.,0.]});
}