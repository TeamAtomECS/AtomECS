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
use crate::integrator::{AddOldForceToNewAtomsSystem, Step, VelocityVerletIntegrationSystem};
use crate::laser;
use crate::laser::repump::Dark;
use crate::magnetic;
use crate::output::console_output::ConsoleOutputSystem;
use crate::sim_region;
use specs::{Dispatcher, DispatcherBuilder, World};

/// Registers all components used by the modules of the program.
pub fn register_components(world: &mut World) {
	atom::register_components(world);
	magnetic::register_components(world);
	laser::register_components(world);
	atom_sources::register_components(world);
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
	builder = builder.with(ClearForceSystem, "clear", &[]);
	builder = builder.with(DeflagNewAtomsSystem, "deflag", &[]);
	builder = magnetic::add_systems_to_dispatch(builder, &[]);
	builder = laser::add_systems_to_dispatch(builder, &[]);
	builder = atom_sources::add_systems_to_dispatch(builder, &[]);
	builder = builder.with(ApplyGravitationalForceSystem, "add_gravity", &["clear"]);
	builder = builder.with(
		VelocityVerletIntegrationSystem,
		"integrator",
		&[
			"calculate_absorption_forces",
			"calculate_emission_forces",
			"add_gravity",
		],
	);
	builder = builder.with(ConsoleOutputSystem, "", &["integrator"]);
	builder = builder.with(DeleteToBeDestroyedEntitiesSystem, "", &["integrator"]);
	builder.add(AddOldForceToNewAtomsSystem, "", &[]);
	builder = sim_region::add_systems_to_dispatch(builder, &[]);
	builder
}

/// Add required resources to the world
pub fn register_resources(world: &mut World) {
	world.add_resource(Step { n: 0 });
}
