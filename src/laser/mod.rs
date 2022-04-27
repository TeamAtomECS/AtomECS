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
    query: Query<Entity, With<NewlyCreated>>
) {
    for ent in query.iter() {
        commands.entity(ent)
        .insert(
            intensity::LaserIntensitySamplers {
                contents: [intensity::LaserIntensitySampler::default(); N],
            }
        )
        .insert(
            intensity_gradient::LaserIntensityGradientSamplers {
                contents: [intensity_gradient::LaserIntensityGradientSampler::default(); N],
            }
        )
        ;
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
pub struct LaserPlugin<const N : usize>;
impl<const N : usize> Plugin for LaserPlugin<N> {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new().label(LaserSystems::Set)
                .with_system(attach_laser_components_to_newly_created_atoms::<N>.label(LaserSystems::AttachLaserComponentsToNewlyCreatedAtoms))
                .with_system(index::index_lasers.label(LaserSystems::IndexLasers).label(LaserSystems::SamplersReady))
                .with_system(intensity::initialise_laser_intensity_samplers::<N>.label(LaserSystems::InitialiseLaserIntensitySamplers).label(LaserSystems::SamplersReady))
                .with_system(intensity::sample_laser_intensities::<N, RequiresIntensityCalculation>.label(LaserSystems::SampleLaserIntensities).after(LaserSystems::SamplersReady))
                .with_system(intensity_gradient::sample_gaussian_laser_intensity_gradient::<N, RequiresIntensityGradientCalculation>.label(LaserSystems::SampleLaserIntensityGradients).after(LaserSystems::SamplersReady))
        );
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum LaserSystems {
    Set,
    AttachLaserComponentsToNewlyCreatedAtoms,
    IndexLasers,
    SamplersReady,
    InitialiseLaserIntensitySamplers,
    SampleLaserIntensities,
    SampleLaserIntensityGradients
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;

    /// Test samplers are added to [NewlyCreated] entities.
    #[test]
    fn test_deflag_new_atoms_system() {
        let mut app = App::new();
        //app.add_plugin(InitiatePlugin);
        
        let test_entity = app.world.spawn().insert(NewlyCreated).id();
        app.update();
        //assert!(!app.world.entity(test_entity).contains::<NewlyCreated>());
    }
}
