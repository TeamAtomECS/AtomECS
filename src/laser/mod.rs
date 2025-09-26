//! Calculation and initialization of laser quantities, eg intensities and indexing.

pub mod frame;
pub mod gaussian;
pub mod index;
pub mod intensity;
pub mod intensity_gradient;

use crate::initiate::NewlyCreated;
use bevy::prelude::*;

pub const DEFAULT_BEAM_LIMIT: usize = 16;

/// Attaches components used for laser calculations to [NewlyCreated] entities.
fn attach_laser_components_to_newly_created_atoms<const N: usize>(
    mut commands: Commands,
    query: Query<Entity, With<NewlyCreated>>,
) {
    for ent in query.iter() {
        commands
            .entity(ent)
            .insert(intensity::LaserIntensitySamplers {
                contents: [intensity::LaserIntensitySampler::default(); N],
            })
            .insert(intensity_gradient::LaserIntensityGradientSamplers {
                contents: [intensity_gradient::LaserIntensityGradientSampler::default(); N],
            });
    }
}

/// Indicates that a laser should be included for intensity calculations.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RequiresIntensityCalculation;
/// Indicates that a laser should be included for intensity gradient calculations.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RequiresIntensityGradientCalculation;

/// This plugin provides basic functionality for laser beams, such as calculating laser intensity.
///
/// See [crate::laser] for more information.
///
/// # Generic Arguments
///
/// * `N`: The maximum number of laser beams to configure the simulation for.
pub struct LaserPlugin<const N: usize>;
impl<const N: usize> Plugin for LaserPlugin<N> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            (
                attach_laser_components_to_newly_created_atoms::<N>,
                index::index_lasers
                    .in_set(LaserSystemsSet::SamplersReady)
                    .in_set(LaserSystemsSet::IndexLasers),
                intensity::initialise_laser_intensity_samplers::<N>
                    .in_set(LaserSystemsSet::SamplersReady),
                intensity::sample_laser_intensities::<N, RequiresIntensityCalculation>
                    .after(LaserSystemsSet::SamplersReady),
                intensity_gradient::sample_gaussian_laser_intensity_gradient::<
                    N,
                    RequiresIntensityGradientCalculation,
                >
                    .after(LaserSystemsSet::SamplersReady),
            )
                .in_set(LaserSystemsSet::Set),
        );
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemSet)]
pub enum LaserSystemsSet {
    Set,
    SamplersReady,
    IndexLasers,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test samplers are added to [NewlyCreated] entities.
    #[test]
    fn test_components_added_to_new_atoms() {
        use crate::{
            integrator::AtomECSBatchStrategy,
            laser::{
                intensity::LaserIntensitySamplers,
                intensity_gradient::LaserIntensityGradientSamplers,
            },
        };
        const LASER_SIZE: usize = 4;

        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());
        app.add_plugins(LaserPlugin::<LASER_SIZE>);

        let test_entity = app.world_mut().spawn(NewlyCreated).id();
        app.update();
        assert!(app
            .world()
            .entity(test_entity)
            .contains::<LaserIntensitySamplers<LASER_SIZE>>());
        assert!(app
            .world()
            .entity(test_entity)
            .contains::<LaserIntensityGradientSamplers<LASER_SIZE>>());
    }
}
