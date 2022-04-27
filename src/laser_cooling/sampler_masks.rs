//! Additional utilities for laser samplers.
extern crate serde;
use crate::{laser::index::LaserIndex, integrator::BatchSize};
use serde::Serialize;
use bevy::{prelude::*, tasks::ComputeTaskPool};
//use crate::laser_cooling::CoolingLight;

/// Tracks which slots in the laser sampler arrays are currently used for cooling light.
#[derive(Clone, Copy, Default, Serialize)]
pub struct CoolingLaserSamplerMask {
    /// Marks whether a cooling light exists for this slot in the laser sampler array.
    pub filled: bool,
}

/// Component that holds a vector of [LaserSamplerMask]
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CoolingLaserSamplerMasks<const N: usize> {
    /// List of `LaserSamplerMask`s
    pub contents: [CoolingLaserSamplerMask; N],
}

/// Marks all [LaserSamplerMasks] as empty.
pub fn initialise_laser_sampler_masks<const N: usize>(
    mut query: Query<&mut CoolingLaserSamplerMasks<N>>,
) {
    for masks in query.iter()
    {
        masks.contents = [CoolingLaserSamplerMask::default(); N];
    }
}

/// Determines which laser sampler slots are currently being used.
pub struct FillLaserSamplerMasksSystem<const N: usize>;

impl<'a, const N: usize> System<'a> for FillLaserSamplerMasksSystem<N> {
    type SystemData = (
        ReadStorage<'a, LaserIndex>,
        ReadStorage<'a, CoolingLight>,
        WriteStorage<'a, CoolingLaserSamplerMasks<N>>,
    );
    fn run(&mut self, (light_index, cooling, mut masks): Self::SystemData) {
        use rayon::prelude::*;

        for (light_index, _) in (&light_index, &cooling).join() {
            (&mut masks).par_join().for_each(|masks| {
                masks.contents[light_index.index] = LaserSamplerMask { filled: true };
            });
        }
    }
}

pub fn fill_laser_sampler_masks<const N: usize>(
    mut query: Query<&mut 
)