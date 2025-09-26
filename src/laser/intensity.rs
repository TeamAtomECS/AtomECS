//! Calculate the intensity of laser beams

use super::frame::Frame;
use super::gaussian::{get_gaussian_beam_intensity, CircularMask, GaussianBeam};
use crate::atom::Position;
use crate::integrator::AtomECSBatchStrategy;
use crate::laser::index::LaserIndex;
use bevy::prelude::*;
use serde::Serialize;

const LASER_CACHE_SIZE: usize = 16;

/// Represents the laser intensity at the position of the atom with respect to a certain laser beam
#[derive(Clone, Copy, Serialize)]
pub struct LaserIntensitySampler {
    /// Intensity in SI units of W/m^2
    pub intensity: f64,
}

impl Default for LaserIntensitySampler {
    fn default() -> Self {
        LaserIntensitySampler {
            // Intensity in SI units of W/m^2
            intensity: f64::NAN,
        }
    }
}

/// Component that holds a list of `LaserIntensitySamplers`
#[derive(Copy, Clone, Serialize, Component)]
pub struct LaserIntensitySamplers<const N: usize> {
    /// List of laser samplers
    #[serde(with = "serde_arrays")]
    pub contents: [LaserIntensitySampler; N],
}

/// This system initialises all `LaserIntensitySamplers` to a NAN value.
///
/// It also ensures that the size of the `LaserIntensitySamplers` components match the number of CoolingLight entities in the world.
///
/// # Generic Arguments
///
/// * `N`: a constant `usize` corresponding to the size of the laser sampler array.
pub fn initialise_laser_intensity_samplers<const N: usize>(
    mut query: Query<&mut LaserIntensitySamplers<N>>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|mut sampler| {
            sampler.contents = [LaserIntensitySampler::default(); N];
        })
}

/// System that calculates the intensity of [GaussianBeam] lasers at the [Position] of each [LaserIntensitySamplers].
///
/// # Generic Arguments
///
/// * `N`: a constant `usize` corresponding to the size of the laser sampler array.
/// * `FilterT`: a component type used to filter which beams intensity will be calculated for, e.g. `CoolingLight`.
pub fn sample_laser_intensities<const N: usize, FilterT>(
    laser_query: Query<(Entity, &LaserIndex, &GaussianBeam), With<FilterT>>,
    mask_query: Query<&CircularMask>,
    frame_query: Query<&Frame>,
    mut sampler_query: Query<(&mut LaserIntensitySamplers<N>, &Position)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) where
    FilterT: Component,
{
    // There are typically only a small number of lasers in a simulation.
    // For a speedup, cache the required components into thread memory,
    // so they can be distributed to parallel workers during the atom loop.
    type CachedLaser = (
        LaserIndex,
        GaussianBeam,
        Option<CircularMask>,
        Option<Frame>,
    );
    let mut laser_cache: Vec<CachedLaser> = Vec::new();
    for (laser_entity, index, gaussian) in laser_query.iter() {
        laser_cache.push((
            *index,
            *gaussian,
            if mask_query.contains(laser_entity) {
                Some(*mask_query.get(laser_entity).unwrap())
            } else {
                None
            },
            if frame_query.contains(laser_entity) {
                Some(*frame_query.get(laser_entity).unwrap())
            } else {
                None
            },
        ));
    }

    // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
    for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
        let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
        let slice = &laser_cache[base_index..max_index];
        let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
        laser_array[..max_index].copy_from_slice(slice);
        let number_in_iteration = slice.len();

        sampler_query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(mut samplers, pos)| {
                for (index, gaussian, mask, frame) in laser_array.iter().take(number_in_iteration) {
                    samplers.contents[index.index].intensity =
                        get_gaussian_beam_intensity(gaussian, pos, mask.as_ref(), frame.as_ref());
                }
            });
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::laser::gaussian;
    use crate::laser::index::LaserIndex;
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;

    #[derive(Component)]
    struct TestComp;

    /// Tests the correct sampling of laser intensities.
    #[test]
    fn test_sample_laser_intensity_system() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());

        app.world_mut()
            .spawn(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(TestComp)
            .insert(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
                rayleigh_range: gaussian::calculate_rayleigh_range(&461.0e-9, &2.0),
                ellipticity: 0.0,
            });

        let atom1 = app
            .world_mut()
            .spawn(Position { pos: Vector3::y() })
            .insert(LaserIntensitySamplers {
                contents: [LaserIntensitySampler::default(); 1],
            })
            .id();

        app.add_systems(Update, sample_laser_intensities::<1, TestComp>);
        app.update();

        let actual_intensity = gaussian::get_gaussian_beam_intensity(
            &GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
                rayleigh_range: gaussian::calculate_rayleigh_range(&461.0e-9, &2.0),
                ellipticity: 0.0,
            },
            &Position { pos: Vector3::y() },
            None,
            None,
        );

        assert_approx_eq!(
            app.world()
                .entity(atom1)
                .get::<LaserIntensitySamplers::<1>>()
                .expect("entity not found")
                .contents[0]
                .intensity,
            actual_intensity,
            1e-6_f64
        );
    }

    /// Tests that laser intensity samplers are reinitialised to zero at the start of the frame.
    #[test]
    fn test_initialise_laser_intensity_samplers() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());

        let atom1 = app
            .world_mut()
            .spawn(Position { pos: Vector3::y() })
            .insert(LaserIntensitySamplers {
                contents: [LaserIntensitySampler { intensity: 1.0 }; 1],
            })
            .id();

        app.add_systems(Update, initialise_laser_intensity_samplers::<1>);
        app.update();

        assert!(app
            .world()
            .entity(atom1)
            .get::<LaserIntensitySamplers::<1>>()
            .expect("entity not found")
            .contents[0]
            .intensity
            .is_nan());
    }
}
