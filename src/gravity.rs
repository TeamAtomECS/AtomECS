//! Implements the force of gravity.

use crate::atom::{Force, Mass};
use crate::constant;
use crate::integrator::AtomECSBatchStrategy;
use bevy::prelude::*;
use nalgebra::Vector3;

/// A resource that indicates that the simulation should apply the force of gravity.
#[derive(Resource)]
#[derive(Default)]
pub struct GravityConfiguration {
    pub apply_gravity: bool,
}


fn apply_gravitational_forces(
    batch_strategy: Res<AtomECSBatchStrategy>,
    config: Res<GravityConfiguration>,
    mut query: Query<(&mut Force, &Mass)>,
) {
    if config.apply_gravity {
        query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(mut force, mass)| {
                force.force +=
                    mass.value * constant::AMU * constant::GC * Vector3::new(0., 0., -1.);
            });
    }
}

/// This plugin implements the force of gravity.
///
/// See also [crate::gravity].
pub struct GravityPlugin;
impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().insert_resource(GravityConfiguration::default());
        app.add_systems(Update, apply_gravitational_forces);
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Tests the correct implementation of the `ApplyGravitationalForceSystem`
    #[test]
    fn test_apply_gravitational_force_system() {
        let mut simulation = App::new();
        simulation.add_plugins(GravityPlugin);
        simulation.insert_resource(AtomECSBatchStrategy::default());
        let atom = simulation
            .world_mut()
            .spawn(Mass { value: 1.0 })
            .insert(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .id();

        // Test gravity off
        simulation.insert_resource(GravityConfiguration {
            apply_gravity: false,
        });
        simulation.update();
        assert_approx_eq!(
            simulation
                .world()
                .get::<Force>(atom)
                .expect("entity not found")
                .force[2],
            0.0,
            1e-30_f64
        );

        // Test gravity on
        simulation.insert_resource(GravityConfiguration {
            apply_gravity: true,
        });
        simulation.update();
        assert_approx_eq!(
            simulation
                .world()
                .get::<Force>(atom)
                .expect("entity not found")
                .force[2],
            -1.0 * constant::AMU * constant::GC,
            1e-30_f64
        );
    }
}
