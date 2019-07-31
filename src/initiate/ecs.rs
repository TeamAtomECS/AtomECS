use crate::atom::{Atom,Mass,Position,Velocity,Force,RandKick,Gravity};
use crate::initiate::{AtomInfo,NewlyCreated};
use crate::laser;
use crate::magnetic;
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
    magnetic::register_resources(&mut world);
    laser::register_resources(&mut world);
}

fn add_systems_to_dispatch_general_update(builder: DispatcherBuilder<'static,'static>, deps: &[&str]) -> DispatcherBuilder<'static,'static> {
	builder.
	with(EulerIntegrationSystem,"updatepos",&["update_kick"]).
	with(PrintOutputSytem,"print",&["updatepos"]).
	with(DetectingAtomSystem,"detect",&["updatepos"])
}

/// Creates a `Dispatcher` that can be used to calculate each simulation frame.
pub fn create_simulation_dispatcher()->Dispatcher<'static,'static>{
    let mut builder = DispatcherBuilder::new();
    builder = magnetic::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
    builder = laser::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
    builder = add_systems_to_dispatch_general_update(builder, &[]);
    builder.build()
}