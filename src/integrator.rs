//! Module that performs time-integration.
//!
//! This module implements the [EulerIntegrationSystem](struct.EulerIntegrationSystem.html),
//! which uses the euler method to integrate classical equations of motion.

extern crate nalgebra;
extern crate rand;
extern crate specs;

use crate::atom::*;
use crate::constant;
use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

/// Tracks the number of the current integration step.
pub struct Step {
	pub n: u64,
}

/// The timestep used for the integration.
///
/// The duration of the timestep should be sufficiently small to resolve the fastest timescale of motion,
/// otherwise significant numerical errors will accumulate during the integration.
/// For a typical magneto-optical trap simulation, the timestep should be around 1us.
/// Decreasing the timestep further will not improve the accuracy, and will require more integration steps
/// to simulate the same total simulation time.
pub struct Timestep {
	/// Duration of the simulation timestep, in SI units of seconds.
	pub delta: f64,
}

/// # Euler Integration
///
/// The EulerIntegrationSystem integrates the classical equations of motion for particles using the euler method:
/// `x' = x + v * dt`.
/// This integrator is simple to implement but prone to integration error.
///
/// The timestep duration is specified by the [Timestep](struct.Timestep.html) system resource.
pub struct EulerIntegrationSystem;

impl<'a> System<'a> for EulerIntegrationSystem {
	type SystemData = (
		WriteStorage<'a, Position>,
		WriteStorage<'a, Velocity>,
		ReadExpect<'a, Timestep>,
		WriteExpect<'a, Step>,
		ReadStorage<'a, Force>,
		ReadStorage<'a, Mass>,
	);

	fn run(&mut self, (mut pos, mut vel, t, mut step, force, mass): Self::SystemData) {
		step.n = step.n + 1;
		for (mut vel, mut pos, force, mass) in (&mut vel, &mut pos, &force, &mass).join() {
			euler_update(&mut vel, &mut pos, &force, &mass, t.delta);
		}
	}
}

/// Performs the euler method to update [Velocity](struct.Velocity.html) and [Position](struct.Position.html) given an applied [Force](struct.Force.html).
fn euler_update(vel: &mut Velocity, pos: &mut Position, force: &Force, mass: &Mass, dt: f64) {
	pos.pos = pos.pos + vel.vel * dt;
	vel.vel = vel.vel + force.force * dt / (constant::AMU * mass.value);
}

pub mod tests {
	#[allow(unused_imports)]
	use super::*;
	extern crate specs;
	#[allow(unused_imports)]
	use specs::{Builder, DispatcherBuilder, World};

	extern crate nalgebra;
	#[allow(unused_imports)]
	use nalgebra::Vector3;

	#[test]
	fn test_euler() {
		let mut pos = Position {
			pos: Vector3::new(1., 1., 1.),
		};
		let mut vel = Velocity {
			vel: Vector3::new(0., 1., 0.),
		};
		let time = 1.;
		let mass = Mass {
			value: 1. / constant::AMU,
		};
		let force = Force {
			force: Vector3::new(1., 1., 1.),
		};
		euler_update(&mut vel, &mut pos, &force, &mass, time);
		assert_eq!(vel.vel, Vector3::new(1., 2., 1.));
		assert_eq!(pos.pos, Vector3::new(1., 2., 1.));
	}

	/// Tests the [EulerIntegrationSystem] by creating a mock world and integrating the trajectory of one entity.
	#[test]
	fn test_euler_system() {
		let mut test_world = World::new();

		let mut dispatcher = DispatcherBuilder::new()
			.with(EulerIntegrationSystem, "integrator", &[])
			.build();
		dispatcher.setup(&mut test_world.res);

		let initial_position = Vector3::new(0.0, 0.1, 0.0);
		let initial_velocity = Vector3::new(1.0, 1.5, 0.4);
		let initial_force = Vector3::new(0.4, 0.6, -0.4);
		let mass = 2.0 / constant::AMU;
		let test_entity = test_world
			.create_entity()
			.with(Position {
				pos: initial_position,
			})
			.with(Velocity {
				vel: initial_velocity,
			})
			.with(Force {
				force: initial_force,
			})
			.with(Mass { value: mass })
			.build();

		let dt = 1.0;
		test_world.add_resource(Timestep { delta: dt });
		test_world.add_resource(Step { n: 0 });

		dispatcher.dispatch(&mut test_world.res);

		let velocities = test_world.read_storage::<Velocity>();
		let velocity = velocities.get(test_entity).expect("entity not found");
		let initial_acceleration = &initial_force / (&mass * constant::AMU);
		assert_eq!(velocity.vel, initial_velocity + initial_acceleration * dt);
		let positions = test_world.read_storage::<Position>();
		let position = positions.get(test_entity).expect("entity not found");
		assert_eq!(position.pos, initial_position + initial_velocity * dt);
	}
}
