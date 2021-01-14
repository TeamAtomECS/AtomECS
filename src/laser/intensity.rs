// !This file exists because - in the spirit of keeping things general - I thought that the intensity sampler should not be in
// gaussian.rs since other beam profiles (although they're less common) should not be excluded.

extern crate rayon;
extern crate specs;

use super::cooling::{CoolingLight, CoolingLightIndex};
use super::gaussian::{get_gaussian_beam_intensity, CircularMask, GaussianBeam};
use crate::atom::Position;
use specs::{Component, Entities, Join, ReadStorage, System, VecStorage, WriteStorage};

const LASER_CACHE_SIZE: usize = 16;

/// Represents the Laser intensity at the position of the atom with respect to a certain laser beam
#[derive(Clone)]
pub struct LaserIntensitySampler {
    pub intensity: f64,
}

impl Default for LaserIntensitySampler {
    fn default() -> Self {
        LaserIntensitySampler {
            /// Doppler shift with respect to laser beam, in SI units of Hz.
            intensity: f64::NAN,
        }
    }
}

/// Component that holds a list of laser intensity samplers
pub struct LaserIntensitySamplers {
    /// List of laser samplers
    pub contents: Vec<LaserIntensitySampler>,
}
impl Component for LaserIntensitySamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all LaserIntensitySamplers to a NAN value.
///
/// It also ensures that the size of the LaserIntensitySamplers components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserIntensitySamplersSystem;
impl<'a> System<'a> for InitialiseLaserIntensitySamplersSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LaserIntensitySamplers>,
    );
    fn run(&mut self, (cooling, cooling_index, mut samplers): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LaserIntensitySampler::default());
        }

        for mut sampler in (&mut samplers).join() {
            sampler.contents = content.clone();
        }
    }
}

/// System that calculates that samples the intensity of Beam entities, for example `GaussianBeam` entities.
pub struct SampleLaserIntensitySystem;
impl<'a> System<'a> for SampleLaserIntensitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, CircularMask>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, LaserIntensitySamplers>,
    );

    fn run(
        &mut self,
        (entities, indices, gaussian, masks, position, mut intensity_samplers): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = (CoolingLightIndex, GaussianBeam, Option<CircularMask>);
        let laser_cache: Vec<CachedLaser> = (&entities, &indices, &gaussian)
            .join()
            .map(|(laser_entity, index, gaussian)| {
                (
                    index.clone(),
                    gaussian.clone(),
                    masks.get(laser_entity).cloned(),
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
                        let (index, gaussian, mask) = laser_array[i];
                        samplers.contents[index.index].intensity =
                            get_gaussian_beam_intensity(&gaussian, &pos, mask.as_ref());
                    }
                });
        }
    }
}
