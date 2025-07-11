//! Masks to describe which lasers are used for [CoolingLight] calculations.

extern crate serde;
use crate::{integrator::AtomECSBatchStrategy, laser::index::LaserIndex};
use bevy::prelude::*;
use serde::Serialize;

use super::CoolingLight;

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

/// Populates [LaserSamplerMasks] as empty or filled.
pub fn populate_cooling_light_masks<const N: usize>(
    mut query: Query<&mut CoolingLaserSamplerMasks<N>>,
    light_query: Query<&LaserIndex, With<CoolingLight>>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    let mut masks = [CoolingLaserSamplerMask::default(); N];
    for index in light_query.iter() {
        masks[index.index] = CoolingLaserSamplerMask { filled: true };
    }

    // distribute the masks into atom components.
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|mut atom_masks| {
            atom_masks.contents = masks;
        });
}
