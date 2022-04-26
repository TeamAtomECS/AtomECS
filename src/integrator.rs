//! Implements systems to integrate trajectories.

use crate::atom::*;
use crate::constant;
use crate::initiate::NewlyCreated;
use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;
use nalgebra::Vector3;

/// Tracks the number of the current integration step.
pub struct Step {
    pub n: u64,
}
impl Default for Step {
    fn default() -> Self {
        Step { n: 0 }
    }
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
impl Default for Timestep {
    fn default() -> Self {
        Timestep { delta: 1.0e-6 }
    }
}

pub const INTEGRATE_POSITION_SYSTEM_NAME: &str = "integrate_position";

pub struct BatchSize(pub usize);
impl Default for BatchSize {
    fn default() -> Self {
        BatchSize(1024)
    }
}

/// Integrates position using a velocity-verlet integration approach.
/// Stores the value of [Force] from the previous frame in the [OldForce] component.
///
/// The timestep duration is specified by the [Timestep] system resource.
fn velocity_verlet_integrate_position(
    pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
    timestep: Res<Timestep>,
    mut step: ResMut<Step>,
    mut query: Query<(&mut Position, &mut OldForce, &Velocity, &Force, &Mass)>,
) {
    step.n += 1;
    let dt = timestep.delta;

    query.par_for_each_mut(&pool, batch_size.0, |(mut pos, mut old_force, vel, force, mass)| {
        pos.pos = pos.pos
            + vel.vel * dt
            + force.force / (constant::AMU * mass.value) / 2.0 * dt * dt;
        old_force.0 = *force;
    });
}

pub const INTEGRATE_VELOCITY_SYSTEM_NAME: &str = "integrate_velocity";

/// Integrates velocity using the velocity-verlet method, and the average of `Force` this frame and `OldForce` from the previous frame.
///
/// The timestep duration is specified by the [Timestep] system resource
fn velocity_verlet_integrate_velocity(
    pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
    timestep: Res<Timestep>,
    mut query: Query<(&mut Velocity, &Force, &OldForce, &Mass)>,
) {
    let dt = timestep.delta;
    query.par_for_each_mut(&pool, batch_size.0, |(mut vel, force, old_force, mass)| {
        vel.vel += (force.force + old_force.0.force) / (constant::AMU * mass.value) / 2.0 * dt;
    });
}

/// Adds [OldForce] components to [NewlyCreated] atoms.
fn add_old_force_to_new_atoms(
    mut commands: Commands,
    query: Query<Entity, (With<NewlyCreated>, Without<OldForce>)>
) {
    for ent in query.iter() {
        commands.entity(ent).insert(OldForce::default());
    }
}

/// Resets force to zero at the start of each simulation step.
fn clear_force(
    mut query: Query<&mut Force>,
    pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
) {
    query.par_for_each_mut(
        &pool, batch_size.0,
        |mut force| {
            force.force = Vector3::new(0.0,0.0,0.0);
        }
    )
}

/// Stores the value of the force calculation from the previous frame.
#[derive(Component, Default)]
pub struct OldForce(Force);

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum IntegrationSystems {
    VelocityVerletIntegratePosition,
    VelocityVerletIntegrateVelocity,
    AddOldForceToNewAtoms,
    ClearForce,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum IntegrationStages {
    BeginIntegration,
    EndIntegration
}

pub struct IntegrationPlugin;
impl Plugin for IntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert_resource(BatchSize::default());
        app.world.insert_resource(Step::default());
        app.world.insert_resource(Timestep::default());
        // Add stages for begin/end integration. Stages are used to guarantee all systems have completed.
        // By default, systems are added to CoreStage::Update. We want our integrator to sandwich either side of these.
        app.add_stage_before(CoreStage::Update, IntegrationStages::BeginIntegration, SystemStage::parallel());
        app.add_stage_after(CoreStage::Update, IntegrationStages::EndIntegration, SystemStage::parallel());
        app.add_system_to_stage(IntegrationStages::BeginIntegration, 
            velocity_verlet_integrate_position.label(IntegrationSystems::VelocityVerletIntegratePosition)
        );
        app.add_system_to_stage(IntegrationStages::BeginIntegration, 
            clear_force.label(IntegrationSystems::ClearForce)
        );
        app.add_system_to_stage(IntegrationStages::BeginIntegration, 
            add_old_force_to_new_atoms.label(IntegrationSystems::AddOldForceToNewAtoms)
        );
        app.add_system_to_stage(IntegrationStages::EndIntegration, 
            velocity_verlet_integrate_velocity.label(IntegrationSystems::VelocityVerletIntegrateVelocity)
        );
    }
}



pub mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_add_old_force_system() {
        let mut app = App::new();
        app.add_plugin(IntegrationPlugin);

        let test_entity = app.world.spawn().insert(NewlyCreated).id();
        app.update();
        assert!(
            app.world.entity(test_entity).contains::<OldForce>(),
            "OldForce component not added to test entity."
        );
    }

    #[test]
    fn test_velocity_verlet_integration() {
        let mut app = App::new();
        app.add_plugin(IntegrationPlugin);

        fn get_force_for_test() -> Vector3<f64> {
            Vector3::new(1.0, 0.0, 0.0)
        }
    
        fn set_force_for_testing(
            mut query: Query<&mut Force>
        ) {
            for mut force in query.iter_mut() {
                force.force = get_force_for_test();
            }
        }

        app.add_system_to_stage(CoreStage::Update, set_force_for_testing);

        // create a particle with known force and mass
        let force = get_force_for_test();
        let mass = 1.0;

        let test_entity = app.world
            .spawn()
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(Force { force })
            .insert(OldForce {
                0: Force { force },
            })
            .insert(Mass {
                value: mass / constant::AMU,
            })
            .id();

        let dt = 1.0e-3;
        app.world.insert_resource(Timestep { delta: dt });
        

        // run simulation loop 1_000 times.
        let n_steps = 1_000;
        for _i in 0..n_steps {
            app.update()
        }

        let a = force / mass;
        let expected_v = a * (n_steps as f64 * dt);
        
        assert_approx_eq::assert_approx_eq!(
            expected_v.norm(),
            app.world.entity(test_entity).get::<Velocity>().expect("test_entity does not have velocity.").vel.norm(),
            expected_v.norm() * 0.01
        );

        let expected_x = a * (n_steps as f64 * dt).powi(2) / 2.0;
        assert_approx_eq::assert_approx_eq!(
            expected_x.norm(),
            app.world.entity(test_entity).get::<Position>().expect("test_entity does not have velocity.").pos.norm(),
            expected_x.norm() * 0.01
        );
    }
}
