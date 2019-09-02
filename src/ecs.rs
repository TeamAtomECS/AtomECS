use crate::atom::ClearForceSystem;
use crate::atom::Index;
use crate::atom_sources;
use crate::destructor::{DeleteToBeDestroyedEntitiesSystem, DestroyOutOfBoundAtomsSystem};
use crate::gravity::ApplyGravitationalForceSystem;
use crate::initiate::DeflagNewAtomsSystem;

use crate::integrator::EulerIntegrationSystem;
use crate::integrator::{Step};

use crate::detector;
use crate::detector::DetectingInfo;

use crate::output::console_output::ConsoleOutputSystem;
use specs::{Dispatcher, DispatcherBuilder, World};

use crate::laser;

use crate::magnetic;
use crate::output::file_output::FileOutputSystem;

use crate::optimization::OptEarlySystem;

use crate::laser::repump::Dark;

extern crate nalgebra;
use nalgebra::Vector3;

/// Registers all components used by the modules of the program.
pub fn register_components(world: &mut World) {
	magnetic::register_components(world);
	laser::register_components(world);
	atom_sources::register_components(world);
	detector::register_components(world);
	world.register::<Dark>();
}

/// Creates a `Dispatcher` that can be used to calculate each simulation frame.
pub fn create_simulation_dispatcher() -> Dispatcher<'static, 'static> {
	let mut builder = DispatcherBuilder::new();
	builder = builder.with(OptEarlySystem, "opt", &[]);
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
	builder = detector::add_systems_to_dispatch(builder, &[]);

	builder = builder.with(ConsoleOutputSystem, "", &["euler_integrator"]);

	builder = builder.with(FileOutputSystem::new("output.txt".to_string(), 10), "", &[]);
	builder = builder.with(
		DeleteToBeDestroyedEntitiesSystem,
		"",
		&["detect_atom", "euler_integrator"],
	);
	//it is quite necessary to put the delete system at the end, otherwise some atoms are not detroyed on time
	builder = builder.with(DestroyOutOfBoundAtomsSystem, "", &[]);
	builder.build()
}

/// Add resources to the world
pub fn register_resources(world: &mut World) {
	world.add_resource(Step { n: 0 });
	world.add_resource(Index { current_index: 0 });
	world.add_resource(DetectingInfo {
		atom_detected: 0,
		total_velocity: Vector3::new(0., 0., 0.),
	});
}
