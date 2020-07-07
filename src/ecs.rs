//! Helper methods to setup the ECS world and dispatcher.
//!
//! This module contains a number of helpful methods that are used to setup the `specs::World`
//! and to create the `specs::Dispatcher` that is used to perform the simulation itself.

use crate::atom;
use crate::atom::ClearForceSystem;
use crate::atom_sources;
use crate::destructor::DeleteToBeDestroyedEntitiesSystem;
//use crate::detector;
//use crate::detector::DetectingInfo;
use crate::gravity::ApplyGravitationalForceSystem;
use crate::initiate::DeflagNewAtomsSystem;
use crate::integrator::{EulerIntegrationSystem, Step};
use crate::laser;
use crate::laser::repump::Dark;
use crate::magnetic;
use crate::optimization::LargerEarlyTimestepOptimizationSystem;
use crate::output::console_output::ConsoleOutputSystem;
use crate::sim_region;
use specs::{Dispatcher, DispatcherBuilder, World};

/// Registers all components used by the modules of the program.
pub fn register_components(world: &mut World) {
	atom::register_components(world);
	magnetic::register_components(world);
	laser::register_components(world);
	atom_sources::register_components(world);
	//detector::register_components(world);
	sim_region::register_components(world);
	world.register::<Dark>();
}

/// Creates a [Dispatcher](specs::Dispatcher) that is used to calculate each simulation frame.
pub fn create_simulation_dispatcher() -> Dispatcher<'static, 'static> {
	let builder = create_simulation_dispatcher_builder();
	builder.build()
}

pub fn create_simulation_dispatcher_builder() -> DispatcherBuilder<'static, 'static> {
	let mut builder = DispatcherBuilder::new();
	builder = builder.with(LargerEarlyTimestepOptimizationSystem, "opt", &[]);
	builder = builder.with(ClearForceSystem, "clear", &[]);
	builder = builder.with(DeflagNewAtomsSystem, "deflag", &[]);
	builder.add_barrier();
	builder = magnetic::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
	builder = laser::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
	builder = atom_sources::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
	builder = builder.with(ApplyGravitationalForceSystem, "add_gravity", &["clear"]);
	builder = builder.with(
		EulerIntegrationSystem,
		"euler_integrator",
		&[
			"calculate_cooling_forces",
			"random_walk_system",
			"add_gravity",
		],
	);
	//builder = detector::add_systems_to_dispatch(builder, &[]);
	builder = builder.with(ConsoleOutputSystem, "", &["euler_integrator"]);
	builder = builder.with(DeleteToBeDestroyedEntitiesSystem, "", &["euler_integrator"]);
	builder = sim_region::add_systems_to_dispatch(builder, &[]);
	builder.add_barrier();
	builder
}

/// Add required resources to the world
pub fn register_resources(world: &mut World) {
	world.add_resource(Step { n: 0 });
}
