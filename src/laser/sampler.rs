//! Calculation of the total detuning for specific atoms and CoolingLight entities

use crate::atom::AtomicTransition;
use crate::constant;
use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
use crate::laser::doppler::DopplerShiftSamplers;
use crate::magnetic::zeeman::ZeemanShiftSampler;
use specs::prelude::*;
use std::f64;
extern crate nalgebra;

/// Tracks whether slots in the laser sampler arrays are currently used.
#[derive(Clone, Copy)]
pub struct LaserSamplerMask {
    /// Marks whether a cooling light exists for this slot in the laser sampler array.
    pub filled: bool,
}
impl Default for LaserSamplerMask {
    fn default() -> Self {
        LaserSamplerMask { filled: false }
    }
}
/// Component that holds a vector of `LaserSamplerMask`
pub struct LaserSamplerMasks {
    /// List of `LaserSamplerMask`s
    pub contents: [LaserSamplerMask; crate::laser::BEAM_LIMIT],
}
impl Component for LaserSamplerMasks {
    type Storage = VecStorage<Self>;
}

/// Marks all laser sampler mask slots as empty.
pub struct InitialiseLaserSamplerMasksSystem;
impl<'a> System<'a> for InitialiseLaserSamplerMasksSystem {
    type SystemData = (WriteStorage<'a, LaserSamplerMasks>,);

    fn run(&mut self, (mut masks,): Self::SystemData) {
        use rayon::prelude::*;

        (&mut masks).par_join().for_each(|mask| {
            mask.contents = [LaserSamplerMask::default(); crate::laser::BEAM_LIMIT];
        });
    }
}

/// Determines which laser sampler slots are currently being used.
pub struct FillLaserSamplerMasksSystem;
impl<'a> System<'a> for FillLaserSamplerMasksSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LaserSamplerMasks>,
    );
    fn run(&mut self, (light_index, mut masks): Self::SystemData) {
        use rayon::prelude::*;

        for light_index in (&light_index).join() {
            (&mut masks).par_join().for_each(|masks| {
                masks.contents[light_index.index] = LaserSamplerMask { filled: true };
            });
        }
    }
}
