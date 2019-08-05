use crate::atom::{Atom,Mass,Position,Velocity,Force,Gravity};
use crate::laser;
use crate::magnetic;
use crate::initiate::{AtomInfo,NewlyCreated,DeflagNewAtomsSystem};
use crate::integrator::{Timestep,Step};
use crate::initiate::atom_create::{Oven};
use crate::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,Dispatcher};
use crate::output::*;
use crate::visual::{RecordPositionSystem,PlotSystem,PositionRecord};
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
}

/// register component for output system
pub fn register_component_output(world: &mut World) {
		world.register::<Detector>();
		world.register::<RingDetector>();
		world.register::<PositionRecord>();
}

/// if you are lazy and have no idea what you want, use this function to register everything
pub fn register_lazy(mut world: &mut World){
    register_component_output(&mut world);
    register_component_otherforce(&mut world);
    register_component_atomcreation(&mut world);
    register_component_general(&mut world);
    magnetic::register_components(&mut world);
    laser::register_components(&mut world);
}

///  add general update system to dispatcher 
fn add_systems_to_dispatch_general_update(builder: DispatcherBuilder<'static,'static>, deps: &[&str]) -> DispatcherBuilder<'static,'static> {
	builder.
	with(EulerIntegrationSystem,"updatepos",deps).
	with(RecordPositionSystem,"recordpos",&["updatepos"]).
	with(DeflagNewAtomsSystem,"deflag",&["updatepos"]).
	with(PrintOutputSytem,"print",&["updatepos"]).
	with(DetectingAtomSystem,"detect",&["updatepos"])
}

/// Creates a `Dispatcher` that can be used to calculate each simulation frame.
pub fn create_simulation_dispatcher()->Dispatcher<'static,'static>{
    let mut builder = DispatcherBuilder::new();
    builder = magnetic::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
    builder = laser::add_systems_to_dispatch(builder, &["add_magnetic_field_samplers"]);
	builder.add_barrier();
    builder = add_systems_to_dispatch_general_update(builder, &["add_cooling_forces"]);
    builder.build()
}

/// add resources that is necessary easily
pub fn register_resources_lazy(mut world: &mut World){
    world.add_resource(Timestep{delta:5e-6});
    world.add_resource(Step{n:0});
	world.add_resource(AtomOuput{number_of_atom:0,total_velocity:[0.,0.,0.]});
}