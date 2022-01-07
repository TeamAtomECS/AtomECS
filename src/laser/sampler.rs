//! Additional utilities for laser samplers.
extern crate serde;
use crate::laser::index::LaserIndex;
use serde::Serialize;
use specs::prelude::*;
extern crate nalgebra;

use crate::laser_cooling::CoolingLight;

/// Tracks which slots in the laser sampler arrays are currently used for cooling light.
#[derive(Clone, Copy, Default, Serialize)]
pub struct LaserSamplerMask {
    /// Marks whether a cooling light exists for this slot in the laser sampler array.
    pub filled: bool,
}
/// Component that holds a vector of `LaserSamplerMask`
pub struct CoolingLaserSamplerMasks<const N: usize> {
    /// List of `LaserSamplerMask`s
    pub contents: [LaserSamplerMask; N],
}
impl<const N: usize> Component for CoolingLaserSamplerMasks<N> {
    type Storage = VecStorage<Self>;
}

/// Marks all laser sampler mask slots as empty.
pub struct InitialiseLaserSamplerMasksSystem<const N: usize>;

impl<'a, const N: usize> System<'a> for InitialiseLaserSamplerMasksSystem<N> {
    type SystemData = (WriteStorage<'a, CoolingLaserSamplerMasks<N>>,);

    fn run(&mut self, (mut masks,): Self::SystemData) {
        use rayon::prelude::*;

        (&mut masks).par_join().for_each(|mask| {
            mask.contents = [LaserSamplerMask::default(); N];
        });
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
