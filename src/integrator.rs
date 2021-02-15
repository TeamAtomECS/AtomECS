//! Module that performs time-integration.
//!
//! This module implements the [EulerIntegrationSystem](struct.EulerIntegrationSystem.html),
//! which uses the euler method to integrate classical equations of motion.

extern crate nalgebra;
extern crate rand;
extern crate specs;

use crate::atom::*;
use crate::constant;
use crate::initiate::NewlyCreated;
use specs::{Component, ReadExpect, ReadStorage, System, VecStorage, WriteExpect, WriteStorage};
use specs::{Entities, Join, LazyUpdate, Read};

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
		use rayon::prelude::*;
		use specs::ParJoin;

		step.n = step.n + 1;
		(&mut vel, &mut pos, &force, &mass).par_join().for_each(
			|(mut vel, mut pos, force, mass)| {
				euler_update(&mut vel, &mut pos, &force, &mass, t.delta);
			},
		);
	}
}

/// # Velocity-Verlet Integration
///
/// This sytem integrates the classical equations of motion for particles using a velocity-verlet method:
/// `x' = x + v * dt`.
/// This integrator is simple to implement but prone to integration error.
///
/// The timestep duration is specified by the [Timestep](struct.Timestep.html) system resource.
pub struct VelocityVerletIntegrationSystem;

impl<'a> System<'a> for VelocityVerletIntegrationSystem {
	type SystemData = (
		WriteStorage<'a, Position>,
		WriteStorage<'a, Velocity>,
		ReadExpect<'a, Timestep>,
		WriteExpect<'a, Step>,
		ReadStorage<'a, Force>,
		WriteStorage<'a, OldForce>,
		ReadStorage<'a, Mass>,
	);

	fn run(
		&mut self,
		(mut pos, mut vel, t, mut step, force, mut oldforce, mass): Self::SystemData,
	) {
		use rayon::prelude::*;
		use specs::ParJoin;

		step.n = step.n + 1;
		let dt = t.delta;

		(&mut pos, &mut vel, &mut oldforce, &force, &mass)
			.par_join()
			.for_each(|(mut pos, mut vel, mut oldforce, force, mass)| {
				pos.pos = pos.pos
					+ vel.vel * dt + oldforce.0.force / (constant::AMU * mass.value) / 2.0
					* dt * dt;
				vel.vel = vel.vel
					+ (force.force + oldforce.0.force) / (mass.value * constant::AMU) / 2.0 * dt;
				oldforce.0 = *force;
			});
	}
}

/// Adds [OldForce](OldForce.struct.html) components to newly created atoms.
pub struct AddOldForceToNewAtomsSystem;

impl<'a> System<'a> for AddOldForceToNewAtomsSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, NewlyCreated>,
		ReadStorage<'a, OldForce>,
		Read<'a, LazyUpdate>,
	);
	fn run(&mut self, (ent, newly_created, oldforce, updater): Self::SystemData) {
		for (ent, _, _) in (&ent, &newly_created, !&oldforce).join() {
			updater.insert(ent, OldForce::default());
		}
	}
}

/// Stores the value of the force calculation from the previous frame.
pub struct OldForce(Force);
impl Component for OldForce {
	type Storage = VecStorage<OldForce>;
}
impl Default for OldForce {
	fn default() -> Self {
		OldForce { 0: Force::new() }
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

	#[test]
	fn test_add_old_force_system() {
		let mut test_world = World::new();

		let mut dispatcher = DispatcherBuilder::new()
			.with(AddOldForceToNewAtomsSystem, "", &[])
			.build();
		dispatcher.setup(&mut test_world.res);
		test_world.register::<OldForce>();

		let test_entity = test_world.create_entity().with(NewlyCreated {}).build();

		dispatcher.dispatch(&mut test_world.res);
		test_world.maintain();

		let old_forces = test_world.read_storage::<OldForce>();
		assert_eq!(
			old_forces.contains(test_entity),
			true,
			"OldForce component not added to test entity."
		);
	}

	#[test]
	fn test_velocity_verlet_system() {
		let mut test_world = World::new();

		let mut dispatcher = DispatcherBuilder::new()
			.with(VelocityVerletIntegrationSystem, "", &[])
			.build();
		dispatcher.setup(&mut test_world.res);

		let p_1 = Vector3::new(0.0, 0.1, 0.0);
		let v_1 = Vector3::new(1.0, 1.5, 0.4);
		let force_2 = Vector3::new(0.4, 0.6, -0.4);
		let force_1 = Vector3::new(0.2, 0.3, -0.4);
		let mass = 2.0 / constant::AMU;
		let test_entity = test_world
			.create_entity()
			.with(Position { pos: p_1 })
			.with(Velocity { vel: v_1 })
			.with(Force { force: force_2 })
			.with(OldForce {
				0: Force { force: force_1 },
			})
			.with(Mass { value: mass })
			.build();

		let dt = 1.0;
		test_world.add_resource(Timestep { delta: dt });
		test_world.add_resource(Step { n: 0 });

		dispatcher.dispatch(&mut test_world.res);

		let velocities = test_world.read_storage::<Velocity>();
		let velocity = velocities.get(test_entity).expect("entity not found");
		let a_1 = &force_1 / (&mass * constant::AMU);
		let a_2 = &force_2 / (&mass * constant::AMU);
		let v_2 = v_1 + (a_1 + a_2) / 2.0 * dt;
		let p_2 = p_1 + v_1 * dt + a_1 / 2.0 * dt * dt;
		assert!(
			(velocity.vel - v_2).norm().abs() < std::f64::EPSILON,
			"velocity incorrect"
		);
		let positions = test_world.read_storage::<Position>();
		let position = positions.get(test_entity).expect("entity not found");
		let p_error = (position.pos - p_2).norm().abs();
		assert!(
			p_error < std::f64::EPSILON,
			"position incorrect: delta={}",
			p_error
		);
	}
}
