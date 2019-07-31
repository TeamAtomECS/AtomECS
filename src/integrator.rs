extern crate rand;
extern crate specs;

use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

pub struct Step {
	pub n: u64,
}

pub struct Timestep {
	pub delta: f64,
}

use crate::atom::*;
use crate::constant;
use crate::maths;

/// # Euler Integration
///
/// The EulerIntegrationSystem integrates the classical equations of motion for particles using the euler method:
/// `x' = x + v * dt`.
/// This integrator is simple to implement but prone to integration error.
///
/// The timestep duration is specified by the ```Timestep``` system resource.
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

fn euler_update(vel: &mut Velocity, pos: &mut Position, force: &Force, mass: &Mass, dt: f64) {
	pos.pos = maths::array_addition(&pos.pos, &maths::array_multiply(&vel.vel, dt));
	vel.vel = maths::array_addition(
		&vel.vel,
		&maths::array_multiply(&force.force, 1.0 * dt / (constant::AMU * mass.value)),
	);
}

pub mod tests {
	// These imports are actually needed! The compiler is getting confused and warning they are not.
	#[allow(unused_imports)]
	use super::*;
	extern crate specs;
	#[allow(unused_imports)]
	use specs::{Builder, DispatcherBuilder, World};

	#[test]
	fn test_euler() {
		let mut pos = Position { pos: [1., 1., 1.] };
		let mut vel = Velocity { vel: [0., 1., 0.] };
		let time = 1.;
		let mass = Mass {
			value: 1. / constant::AMU,
		};
		let force = Force {
			force: [1., 1., 1.],
		};
		euler_update(&mut vel, &mut pos, &force, &mass, time);
		assert_eq!(vel.vel, [1., 2., 1.]);
		assert_eq!(pos.pos, [1., 2., 1.]);
	}

	/// Tests the [EulerIntegrationSystem] by creating a mock world and integrating the trajectory of one entity.
	#[test]
	fn test_euler_system() {
		let mut test_world = World::new();

		let mut dispatcher = DispatcherBuilder::new()
			.with(EulerIntegrationSystem, "integrator", &[])
			.build();
		dispatcher.setup(&mut test_world.res);

		let initial_position = [0.0, 0.1, 0.0];
		let initial_velocity = [1.0, 1.5, 0.4];
		let initial_force = [0.4, 0.6, -0.4];
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
		let initial_acceleration =
			maths::array_multiply(&initial_force, 1.0 / (&mass * constant::AMU));
		assert_eq!(
			velocity.vel,
			maths::array_addition(
				&initial_velocity,
				&maths::array_multiply(&initial_acceleration, dt)
			)
		);
		let positions = test_world.read_storage::<Position>();
		let position = positions.get(test_entity).expect("entity not found");
		assert_eq!(
			position.pos,
			maths::array_addition(
				&initial_position,
				&maths::array_multiply(&initial_velocity, dt)
			)
		);
	}

}
