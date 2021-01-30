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

/// Struct that creates the ECS Dispatcher builder used in AtomECS.
pub struct AtomecsDispatcherBuilder {
	pub builder: DispatcherBuilder<'static, 'static>,
}
impl AtomecsDispatcherBuilder {
	pub fn new() -> AtomecsDispatcherBuilder {
		AtomecsDispatcherBuilder {
			builder: DispatcherBuilder::new(),
		}
	}

	pub fn add_frame_initialisation_systems(&mut self) {
		&self.builder.add(ClearForceSystem, "clear", &[]);
		&self.builder.add(DeflagNewAtomsSystem, "deflag", &[]);
		&self.builder.add_barrier();
	}

	pub fn add_systems(&mut self) {
		magnetic::add_systems_to_dispatch(&mut self.builder, &[]);
		self.builder.add_barrier();
		laser::add_systems_to_dispatch(&mut self.builder, &[]);
		self.builder.add_barrier();
		atom_sources::add_systems_to_dispatch(&mut self.builder, &[]);
		self.builder.add_barrier();
		self.builder
			.add(ApplyGravitationalForceSystem, "add_gravity", &["clear"]);
	}

	pub fn add_integration_systems(&mut self) {
		&self
			.builder
			.add(EulerIntegrationSystem, "euler_integrator", &["add_gravity"]);
	}

	pub fn add_frame_end_systems(&mut self) {
		&self
			.builder
			.add(ConsoleOutputSystem, "", &["euler_integrator"]);
		&self
			.builder
			.add(DeleteToBeDestroyedEntitiesSystem, "", &["euler_integrator"]);
		sim_region::add_systems_to_dispatch(&mut self.builder, &[]);
		self.builder.add_barrier();
	}

	pub fn build(mut self) -> DispatcherBuilder<'static, 'static> {
		self.add_frame_initialisation_systems();
		self.add_systems();
		self.add_integration_systems();
		self.add_frame_end_systems();
		self.builder
	}
}

/// Creates a [Dispatcher](specs::Dispatcher) that is used to calculate each simulation frame.
pub fn create_simulation_dispatcher() -> Dispatcher<'static, 'static> {
	let builder = create_simulation_dispatcher_builder();
	builder.build()
}

pub fn create_simulation_dispatcher_builder() -> DispatcherBuilder<'static, 'static> {
	let atomecs_builder = AtomecsDispatcherBuilder::new();
	atomecs_builder.build()
}

/// Add required resources to the world
pub fn register_resources(world: &mut World) {
	world.add_resource(Step { n: 0 });
}
