//! Implements systems to integrate trajectories.

use crate::atom::*;
use crate::constant;
use crate::initiate::NewlyCreated;
use bevy::ecs::batching::BatchingStrategy;
use bevy::prelude::*;
use nalgebra::Vector3;

/// Tracks the number of the current integration step.
#[derive(Resource, Default)]
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
#[derive(Resource)]
pub struct Timestep {
    /// Duration of the simulation timestep, in SI units of seconds.
    pub delta: f64,
}
impl Default for Timestep {
    fn default() -> Self {
        Timestep { delta: 1.0e-6 }
    }
}

#[derive(Resource, Clone)]
pub struct AtomECSBatchStrategy(pub BatchingStrategy);
impl Default for AtomECSBatchStrategy {
    fn default() -> Self {
        AtomECSBatchStrategy(BatchingStrategy::fixed(1024))
    }
}

/// Integrates position using a velocity-verlet integration approach.
/// Stores the value of [Force] from the previous frame in the [OldForce] component.
///
/// The timestep duration is specified by the [Timestep] system resource.
fn velocity_verlet_integrate_position(
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
    mut step: ResMut<Step>,
    mut query: Query<(&mut Position, &mut OldForce, &Velocity, &Force, &Mass)>,
) {
    step.n += 1;
    let dt = timestep.delta;

    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut pos, mut old_force, vel, force, mass)| {
            pos.pos =
                pos.pos + vel.vel * dt + force.force / (constant::AMU * mass.value) / 2.0 * dt * dt;
            old_force.0 = *force;
        });
}

/// Integrates velocity using the velocity-verlet method, and the average of `Force` this frame and `OldForce` from the previous frame.
///
/// The timestep duration is specified by the [Timestep] system resource
fn velocity_verlet_integrate_velocity(
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
    mut query: Query<(&mut Velocity, &Force, &OldForce, &Mass)>,
) {
    let dt = timestep.delta;
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut vel, force, old_force, mass)| {
            vel.vel += (force.force + old_force.0.force) / (constant::AMU * mass.value) / 2.0 * dt;
        });
}

/// Adds [OldForce] components to [NewlyCreated] atoms.
fn add_old_force_to_new_atoms(
    mut commands: Commands,
    query: Query<Entity, (With<NewlyCreated>, Without<OldForce>)>,
) {
    for ent in query.iter() {
        commands.entity(ent).insert(OldForce::default());
    }
}

/// Resets force to zero at the start of each simulation step.
fn clear_force(mut query: Query<&mut Force>, batch_strategy: Res<AtomECSBatchStrategy>) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|mut force| {
            force.force = Vector3::new(0.0, 0.0, 0.0);
        });
    }

/// Stores the value of the force calculation from the previous frame.
#[derive(Component, Default)]
pub struct OldForce(pub Force);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum IntegrationSet {
    IntegrationSystems,
    BeginIntegration,
    EndIntegration,
}

pub struct IntegrationPlugin;
impl Plugin for IntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().insert_resource(AtomECSBatchStrategy::default());
        app.world_mut().insert_resource(Step::default());
        app.world_mut().insert_resource(Timestep::default());
        // By default, systems are added to CoreSet::Update. We want our integrator to sandwich either side of these.

        app.configure_sets(PreUpdate,
            IntegrationSet::BeginIntegration,
        );
        app.configure_sets(PostUpdate,
            IntegrationSet::EndIntegration,
        );
        app.add_systems(PreUpdate, velocity_verlet_integrate_position.in_set(IntegrationSet::BeginIntegration));
        app.add_systems(PreUpdate, 
            clear_force
                .in_set(IntegrationSet::BeginIntegration)
                .after(velocity_verlet_integrate_position),
        );
        app.add_systems(PreUpdate, add_old_force_to_new_atoms.in_set(IntegrationSet::BeginIntegration));
        app.add_systems(PostUpdate, velocity_verlet_integrate_velocity.in_set(IntegrationSet::EndIntegration));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_old_force_system() {
        let mut app = App::new();
        app.add_plugins(IntegrationPlugin);

        let test_entity = app.world_mut().spawn(NewlyCreated).id();
        app.update();
        assert!(
            app.world().entity(test_entity).contains::<OldForce>(),
            "OldForce component not added to test entity."
        );
    }

    #[test]
    fn test_velocity_verlet_integration() {
        let mut app = App::new();
        app.add_plugins(IntegrationPlugin);

        fn get_force_for_test() -> Vector3<f64> {
            Vector3::new(1.0, 0.0, 0.0)
        }

        fn set_force_for_testing(mut query: Query<&mut Force>) {
            for mut force in query.iter_mut() {
                force.force = get_force_for_test();
            }
        }

        app.add_systems(Update, set_force_for_testing);

        // create a particle with known force and mass
        let force = get_force_for_test();
        let mass = 1.0;

        let test_entity = app
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(Force { force })
            .insert(OldForce(Force { force }))
            .insert(Mass {
                value: mass / constant::AMU,
            })
            .id();

        let dt = 1.0e-3;
        app.world_mut().insert_resource(Timestep { delta: dt });

        // run simulation loop 1_000 times.
        let n_steps = 1000;
        for _i in 0..n_steps {
            app.update()
        }

        let a = force / mass;
        let expected_v = a * (n_steps as f64 * dt);

        assert_approx_eq::assert_approx_eq!(
            expected_v.norm(),
            app.world()
                .entity(test_entity)
                .get::<Velocity>()
                .expect("test_entity does not have velocity.")
                .vel
                .norm(),
            expected_v.norm() * 0.01
        );

        let expected_x = a * (n_steps as f64 * dt).powi(2) / 2.0;
        assert_approx_eq::assert_approx_eq!(
            expected_x.norm(),
            app.world()
                .entity(test_entity)
                .get::<Position>()
                .expect("test_entity does not have velocity.")
                .pos
                .norm(),
            expected_x.norm() * 0.01
        );
    }
}
