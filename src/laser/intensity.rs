//! Calculation of the intensity of CoolingLight entities at a specific position
//!

// This file exists because - in the spirit of keeping things general - I thought that the intensity sampler should not be in
// gaussian.rs since other beam profiles (although they're less common) should not be excluded.

extern crate rayon;
extern crate specs;

use super::cooling::CoolingLightIndex;
use super::gaussian::{get_gaussian_beam_intensity, CircularMask, GaussianBeam};
use crate::atom::Position;
use crate::laser::gaussian::GaussianRayleighRange;
use specs::{Component, Entities, Join, ReadStorage, System, VecStorage, WriteStorage};

const LASER_CACHE_SIZE: usize = 16;

/// Represents the laser intensity at the position of the atom with respect to a certain laser beam
#[derive(Clone, Copy)]
pub struct LaserIntensitySampler {
    /// Intensity in SI units of W/m^2
    pub intensity: f64,
}

impl Default for LaserIntensitySampler {
    fn default() -> Self {
        LaserIntensitySampler {
            /// Intensity in SI units of W/m^2
            intensity: f64::NAN,
        }
    }
}

/// Component that holds a list of `LaserIntensitySamplers`
pub struct LaserIntensitySamplers {
    /// List of laser samplers
    pub contents: [LaserIntensitySampler; crate::laser::COOLING_BEAM_LIMIT],
}
impl Component for LaserIntensitySamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `LaserIntensitySamplers` to a NAN value.
///
/// It also ensures that the size of the `LaserIntensitySamplers` components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserIntensitySamplersSystem;
impl<'a> System<'a> for InitialiseLaserIntensitySamplersSystem {
    type SystemData = (WriteStorage<'a, LaserIntensitySamplers>,);
    fn run(&mut self, (mut samplers,): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut samplers).par_join().for_each(|mut sampler| {
            sampler.contents = [LaserIntensitySampler::default(); crate::laser::COOLING_BEAM_LIMIT];
        });
    }
}

/// System that calculates the intensity of CoolingLight entities, for example those with `GaussianBeam` components.
///
/// So far, the only intensity distribution implemented as a component for the use
/// along with `CoolingLight` is `GaussianBeam`.
/// However, in the future, other components will be implemented and this System can then be expanded
/// to handle them as well.
pub struct SampleLaserIntensitySystem;
impl<'a> System<'a> for SampleLaserIntensitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, CircularMask>,
        ReadStorage<'a, GaussianRayleighRange>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, LaserIntensitySamplers>,
    );

    fn run(
        &mut self,
        (entities, indices, gaussian, masks, rayleigh_range, position, mut intensity_samplers): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = (
            CoolingLightIndex,
            GaussianBeam,
            Option<CircularMask>,
            Option<GaussianRayleighRange>,
        );
        let laser_cache: Vec<CachedLaser> = (&entities, &indices, &gaussian)
            .join()
            .map(|(laser_entity, index, gaussian)| {
                (
                    index.clone(),
                    gaussian.clone(),
                    masks.get(laser_entity).cloned(),
                    rayleigh_range.get(laser_entity).cloned(),
                )
            })
            .collect();

        // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
        for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
            let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
            let slice = &laser_cache[base_index..max_index];
            let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
            laser_array[..max_index].copy_from_slice(slice);
            let number_in_iteration = slice.len();

            (&mut intensity_samplers, &position)
                .par_join()
                .for_each(|(samplers, pos)| {
                    for i in 0..number_in_iteration {
                        let (index, gaussian, mask, range) = laser_array[i];
                        samplers.contents[index.index].intensity = get_gaussian_beam_intensity(
                            &gaussian,
                            &pos,
                            mask.as_ref(),
                            range.as_ref(),
                        );
                    }
                });
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::laser::cooling::CoolingLightIndex;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Tests the correct implementation of the `SampleLaserIntensitySystem`
    #[test]
    fn test_sample_laser_intensity_system() {
        let mut test_world = World::new();

        test_world.register::<CoolingLightIndex>();
        test_world.register::<GaussianBeam>();
        test_world.register::<GaussianRayleighRange>();
        test_world.register::<CircularMask>();
        test_world.register::<Position>();
        test_world.register::<LaserIntensitySamplers>();

        test_world
            .create_entity()
            .with(CoolingLightIndex {
                index: 0,
                initiated: true,
            })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(Position { pos: Vector3::y() })
            .with(LaserIntensitySamplers {
                contents: [LaserIntensitySampler::default(); crate::laser::COOLING_BEAM_LIMIT],
            })
            .build();

        let mut system = SampleLaserIntensitySystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserIntensitySamplers>();

        let actual_intensity = crate::laser::gaussian::get_gaussian_beam_intensity(
            &GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
            },
            &Position { pos: Vector3::y() },
            None,
            None,
        );

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .contents[0]
                .intensity,
            actual_intensity,
            1e-6_f64
        );
    }
}
