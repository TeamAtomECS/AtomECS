//! Module that performs time-integration.
//!
//! This module implements the [EulerIntegrationSystem](struct.EulerIntegrationSystem.html),
//! which uses the euler method to integrate classical equations of motion.

extern crate nalgebra;

use crate::atom::*;
use crate::constant;
use crate::initiate::NewlyCreated;
use specs::prelude::*;

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

        step.n += 1;
        (&mut vel, &mut pos, &force, &mass).par_join().for_each(
            |(vel, pos, force, mass)| {
                euler_update(vel, pos, force, mass, t.delta);
            },
        );
    }
}

pub const INTEGRATE_POSITION_SYSTEM_NAME: &str = "integrate_position";

/// # Velocity-Verlet Integrate Position
///
/// Integrates position using a velocity-verlet integration approach.
/// Stores the value of `Force` from the previous frame in the `OldForce` component.
///
/// The timestep duration is specified by the [Timestep](struct.Timestep.html) system resource.
pub struct VelocityVerletIntegratePositionSystem;
impl<'a> System<'a> for VelocityVerletIntegratePositionSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadExpect<'a, Timestep>,
        WriteExpect<'a, Step>,
        ReadStorage<'a, Force>,
        WriteStorage<'a, OldForce>,
        ReadStorage<'a, Mass>,
    );

    fn run(&mut self, (mut pos, vel, t, mut step, force, mut old_force, mass): Self::SystemData) {
        use rayon::prelude::*;

        step.n += 1;
        let dt = t.delta;

        (&mut pos, &vel, &mut old_force, &force, &mass)
            .par_join()
            .for_each(|(mut pos, vel, mut old_force, force, mass)| {
                pos.pos = pos.pos
                    + vel.vel * dt
                    + force.force / (constant::AMU * mass.value) / 2.0 * dt * dt;
                old_force.0 = *force;
            });
    }
}

pub const INTEGRATE_VELOCITY_SYSTEM_NAME: &str = "integrate_velocity";

/// # Velocity-Verlet Integrate Velocity
///
/// Integrates velocity using the velocity-verlet method, and the average of `Force` this frame and `OldForce` from the previous frame.
///
/// The timestep duration is specified by the [Timestep](struct.Timestep.html) system resource
pub struct VelocityVerletIntegrateVelocitySystem;
impl<'a> System<'a> for VelocityVerletIntegrateVelocitySystem {
    type SystemData = (
        WriteStorage<'a, Velocity>,
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, Force>,
        ReadStorage<'a, OldForce>,
        ReadStorage<'a, Mass>,
    );

    fn run(&mut self, (mut vel, t, force, old_force, mass): Self::SystemData) {
        use rayon::prelude::*;

        let dt = t.delta;

        (&mut vel, &force, &old_force, &mass).par_join().for_each(
            |(vel, force, old_force, mass)| {
                vel.vel += (force.force + old_force.0.force) / (constant::AMU * mass.value) / 2.0 * dt;
            },
        );
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
    fn run(&mut self, (ent, newly_created, old_force, updater): Self::SystemData) {
        for (ent, _, _) in (&ent, &newly_created, !&old_force).join() {
            updater.insert(ent, OldForce::default());
        }
    }
}

/// Stores the value of the force calculation from the previous frame.
#[derive(Default)]
pub struct OldForce(Force);
impl Component for OldForce {
    type Storage = VecStorage<OldForce>;
}

/// Performs the euler method to update [Velocity](struct.Velocity.html) and [Position](struct.Position.html) given an applied [Force](struct.Force.html).
fn euler_update(vel: &mut Velocity, pos: &mut Position, force: &Force, mass: &Mass, dt: f64) {
    pos.pos += vel.vel * dt;
    vel.vel += force.force * dt / (constant::AMU * mass.value);
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
    fn test_euler_integration() {
        let mut world = World::new();

        let mut dispatcher = DispatcherBuilder::new()
            .with(EulerIntegrationSystem, "integrator", &[])
            .build();
        dispatcher.setup(&mut world);

        // create a particle with known force and mass
        let force = Vector3::new(1.0, 0.0, 0.0);
        let mass = 1.0;
        let atom = world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Force { force })
            .with(Mass {
                value: mass / constant::AMU,
            })
            .build();

        let dt = 1.0e-3;
        world.insert(Timestep { delta: dt });
        world.insert(Step { n: 0 });

        // run simulation loop 1_000 times.
        let n_steps = 1_000;
        for _i in 0..n_steps {
            dispatcher.dispatch(&world);
            world.maintain();
        }

        let a = force / mass;
        let expected_v = a * (n_steps as f64 * dt);

        assert_approx_eq::assert_approx_eq!(
            expected_v.norm(),
            world
                .read_storage::<Velocity>()
                .get(atom)
                .expect("atom not found.")
                .vel
                .norm(),
            expected_v.norm() * 0.01
        );

        let expected_x = a * (n_steps as f64 * dt).powi(2) / 2.0;
        assert_approx_eq::assert_approx_eq!(
            expected_x.norm(),
            world
                .read_storage::<Position>()
                .get(atom)
                .expect("atom not found.")
                .pos
                .norm(),
            expected_x.norm() * 0.01
        );
    }

    #[test]
    fn test_add_old_force_system() {
        let mut test_world = World::new();

        let mut dispatcher = DispatcherBuilder::new()
            .with(AddOldForceToNewAtomsSystem, "", &[])
            .build();
        dispatcher.setup(&mut test_world);
        test_world.register::<OldForce>();

        let test_entity = test_world.create_entity().with(NewlyCreated {}).build();

        dispatcher.dispatch(&test_world);
        test_world.maintain();

        let old_forces = test_world.read_storage::<OldForce>();
        assert!(
            old_forces.contains(test_entity),
            "OldForce component not added to test entity."
        );
    }

    #[test]
    fn test_velocity_verlet_integration() {
        let mut world = World::new();

        let mut dispatcher = DispatcherBuilder::new()
            .with(
                VelocityVerletIntegratePositionSystem,
                "integrate_position",
                &[],
            )
            .with(
                VelocityVerletIntegrateVelocitySystem,
                "integrate_velocity",
                &["integrate_position"],
            )
            .build();
        dispatcher.setup(&mut world);

        // create a particle with known force and mass
        let force = Vector3::new(1.0, 0.0, 0.0);
        let mass = 1.0;
        let atom = world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Force { force })
            .with(OldForce {
                0: Force { force },
            })
            .with(Mass {
                value: mass / constant::AMU,
            })
            .build();

        let dt = 1.0e-3;
        world.insert(Timestep { delta: dt });
        world.insert(Step { n: 0 });

        // run simulation loop 1_000 times.
        let n_steps = 1_000;
        for _i in 0..n_steps {
            dispatcher.dispatch(&world);
            world.maintain();
        }

        let a = force / mass;
        let expected_v = a * (n_steps as f64 * dt);

        assert_approx_eq::assert_approx_eq!(
            expected_v.norm(),
            world
                .read_storage::<Velocity>()
                .get(atom)
                .expect("atom not found.")
                .vel
                .norm(),
            expected_v.norm() * 0.01
        );

        let expected_x = a * (n_steps as f64 * dt).powi(2) / 2.0;
        assert_approx_eq::assert_approx_eq!(
            expected_x.norm(),
            world
                .read_storage::<Position>()
                .get(atom)
                .expect("atom not found.")
                .pos
                .norm(),
            expected_x.norm() * 0.01
        );
    }
}
