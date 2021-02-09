//! Calculation of the total detuning for specific atoms and CoolingLight entities

extern crate specs;
use crate::atom::AtomicTransition;
use crate::constant;
use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
use crate::laser::doppler::DopplerShiftSamplers;
use crate::magnetic::zeeman::ZeemanShiftSampler;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};
use std::f64;
extern crate nalgebra;

const LASER_CACHE_SIZE: usize = 16;

/// Represents total detuning of the atom's transition with respect to each beam
#[derive(Clone)]
pub struct LaserDetuningSampler {
    /// Laser detuning of the sigma plus transition with respect to laser beam, in SI units of Hz
    pub detuning_sigma_plus: f64,
    /// Laser detuning of the sigma minus transition with respect to laser beam, in SI units of Hz
    pub detuning_sigma_minus: f64,
    /// Laser detuning of the pi transition with respect to laser beam, in SI units of Hz
    pub detuning_pi: f64,
}

impl Default for LaserDetuningSampler {
    fn default() -> Self {
        LaserDetuningSampler {
            /// Laser detuning of the sigma plus transition with respect to laser beam, in SI units of Hz
            detuning_sigma_plus: f64::NAN,
            /// Laser detuning of the sigma minus transition with respect to laser beam, in SI units of Hz
            detuning_sigma_minus: f64::NAN,
            /// Laser detuning of the pi transition with respect to laser beam, in SI units of Hz
            detuning_pi: f64::NAN,
        }
    }
}

/// Component that holds a vector of `LaserDetuningSampler`
pub struct LaserDetuningSamplers {
    /// List of `LaserDetuningSampler`s
    pub contents: Vec<LaserDetuningSampler>,
}
impl Component for LaserDetuningSamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `LaserDetuningSamplers` to a NAN value.
///
/// It also ensures that the size of the `LaserDetuningSamplers` components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserDetuningSamplersSystem;
impl<'a> System<'a> for InitialiseLaserDetuningSamplersSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LaserDetuningSamplers>,
    );
    fn run(&mut self, (cooling, cooling_index, mut samplers): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LaserDetuningSampler::default());
        }

        for mut sampler in (&mut samplers).join() {
            sampler.contents = content.to_vec();
        }
    }
}

/// This system calculates the total Laser Detuning for each atom with respect to
/// each CoolingLight entities.
pub struct CalculateLaserDetuningSystem;
impl<'a> System<'a> for CalculateLaserDetuningSystem {
    type SystemData = (
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, DopplerShiftSamplers>,
        ReadStorage<'a, ZeemanShiftSampler>,
        WriteStorage<'a, LaserDetuningSamplers>,
    );

    fn run(
        &mut self,
        (
            atom_info,
            indices,
            cooling_light,
            doppler_samplers,
            zeeman_sampler,
            mut detuning_samplers,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = (CoolingLightIndex, CoolingLight);
        let laser_cache: Vec<CachedLaser> = (&indices, &cooling_light)
            .join()
            .map(|(index, cooling)| (index.clone(), cooling.clone()))
            .collect();

        // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
        for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
            let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
            let slice = &laser_cache[base_index..max_index];
            let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
            laser_array[..max_index].copy_from_slice(slice);
            let number_in_iteration = slice.len();

            (
                &mut detuning_samplers,
                &doppler_samplers,
                &zeeman_sampler,
                &atom_info,
            )
                .par_join()
                .for_each(
                    |(detuning_sampler, doppler_samplers, zeeman_sampler, atom_info)| {
                        for i in 0..number_in_iteration {
                            let (index, cooling) = laser_array[i];
                            let without_zeeman = (constant::C / cooling.wavelength
                                - atom_info.frequency)
                                * 2.0
                                * constant::PI
                                - doppler_samplers.contents[index.index].doppler_shift;

                            detuning_sampler.contents[index.index].detuning_sigma_plus =
                                without_zeeman.clone() - zeeman_sampler.sigma_plus;
                            detuning_sampler.contents[index.index].detuning_sigma_minus =
                                without_zeeman.clone() - zeeman_sampler.sigma_minus;
                            detuning_sampler.contents[index.index].detuning_pi =
                                without_zeeman.clone() - zeeman_sampler.sigma_pi;
                        }
                    },
                )
        }
    }
}
