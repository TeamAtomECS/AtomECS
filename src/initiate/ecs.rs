use crate::atom::{Atom,Mass,Position,Velocity,Force,RandKick,Gravity};
use crate::initiate::{AtomInfo,NewlyCreated};
use crate::update::*;
use crate::laser::*;
use crate::magnetic::*;
use crate::initiate::atom_create::{Oven};
use crate::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,Dispatcher};
use crate::output::*;
pub fn register_resources_general(world: &mut World) {

		world.register::<Position>();
		world.register::<Velocity>();
		world.register::<Force>();
		world.register::<Mass>();
}
pub fn register_resources_atomcreation(world: &mut World) {

		world.register::<Oven>();
		world.register::<AtomInfo>();
        world.register::<Atom>();
		world.register::<NewlyCreated>();
}
pub fn register_resource_otherforce(world: &mut World) {
		world.register::<Gravity>();
		world.register::<RandKick>();
}

pub fn register_resource_output(world: &mut World) {
		world.register::<Detector>();
		world.register::<RingDetector>();
}

pub fn register_lazy(mut world: &mut World){
    register_resource_output(&mut world);
    register_resource_otherforce(&mut world);
    register_resources_atomcreation(&mut world);
    register_resources_general(&mut world);
    register_resources_magnetic(&mut world);
    register_resources_laser(&mut world);

}

fn add_systems_to_dispatch_general_update(builder: DispatcherBuilder<'static,'static>, deps: &[&str]) -> DispatcherBuilder<'static,'static> {
	builder.
	with(UpdateRandKick,"update_kick",deps).
	with(UpdateForce,"updateforce",&["update_kick"]).
	with(EulerIntegrationSystem,"updatepos",&["update_kick"]).
	with(PrintOutputSytem,"print",&["updatepos"]).
	with(DetectingAtomSystem,"detect",&["updatepos"])
}

pub fn create_dispatcher_running()->Dispatcher<'static,'static>{
    let builder=DispatcherBuilder::new();
    let builder_1 = add_systems_to_dispatch_magnetic(builder, &[]);
    let builder_2 = add_systems_to_dispatch_laser(builder_1, &["magnetics_magnitude"]);
    let builder_3 = add_systems_to_dispatch_general_update(builder_2, &["updatelaserinter"]);
    let mut dispatcher = builder_3.build();
    dispatcher
}