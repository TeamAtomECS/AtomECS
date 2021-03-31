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
    pub contents: [LaserSamplerMask; crate::laser::COOLING_BEAM_LIMIT],
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
        use specs::ParJoin;

        (&mut masks).par_join().for_each(|mask| {
            mask.contents = [LaserSamplerMask::default(); crate::laser::COOLING_BEAM_LIMIT];
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
        use specs::ParJoin;

        for light_index in (&light_index).join() {
            (&mut masks).par_join().for_each(|masks| {
                masks.contents[light_index.index] = LaserSamplerMask { filled: true };
            });
        }
    }
}

/// Represents total detuning of the atom's transition with respect to each beam
#[derive(Clone, Copy)]
pub struct LaserDetuningSampler {
    /// Laser detuning of the sigma plus transition with respect to laser beam, in SI units of rad/s
    pub detuning_sigma_plus: f64,
    /// Laser detuning of the sigma minus transition with respect to laser beam, in SI units of rad/s 
    pub detuning_sigma_minus: f64,
    /// Laser detuning of the pi transition with respect to laser beam, in SI units of rad/s
    pub detuning_pi: f64,
}

impl Default for LaserDetuningSampler {
    fn default() -> Self {
        LaserDetuningSampler {
            detuning_sigma_plus: f64::NAN,
            detuning_sigma_minus: f64::NAN,
            detuning_pi: f64::NAN,
        }
    }
}

/// Component that holds a vector of `LaserDetuningSampler`
pub struct LaserDetuningSamplers {
    /// List of `LaserDetuningSampler`s
    pub contents: [LaserDetuningSampler; crate::laser::COOLING_BEAM_LIMIT],
}
impl Component for LaserDetuningSamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `LaserDetuningSamplers` to a NAN value.
///
/// It also ensures that the size of the `LaserDetuningSamplers` components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserDetuningSamplersSystem;
impl<'a> System<'a> for InitialiseLaserDetuningSamplersSystem {
    type SystemData = (WriteStorage<'a, LaserDetuningSamplers>,);
    fn run(&mut self, (mut samplers,): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut samplers).par_join().for_each(|mut sampler| {
            sampler.contents = [LaserDetuningSampler::default(); crate::laser::COOLING_BEAM_LIMIT];
        });
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
                            let without_zeeman = 2.0 * constant::PI * (constant::C / cooling.wavelength - atom_info.frequency)
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

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;

    #[test]
    fn test_calculate_laser_detuning_system() {
        let mut test_world = World::new();
        test_world.register::<CoolingLight>();
        test_world.register::<CoolingLightIndex>();
        test_world.register::<DopplerShiftSamplers>();
        test_world.register::<LaserDetuningSamplers>();
        test_world.register::<AtomicTransition>();
        test_world.register::<ZeemanShiftSampler>();

        let wavelength = constant::C / AtomicTransition::strontium().frequency;
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength: wavelength,
            })
            .with(CoolingLightIndex {
                index: 0,
                initiated: true,
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(DopplerShiftSamplers {
                contents: [crate::laser::doppler::DopplerShiftSampler {
                    doppler_shift: 10.0e6, //rad/s
                }; crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(AtomicTransition::strontium())
            .with(ZeemanShiftSampler {
                sigma_plus: 10.0e6,   //rad/s
                sigma_minus: -10.0e6, //rad/s
                sigma_pi: 0.0,        //rad/s
            })
            .with(LaserDetuningSamplers {
                contents: [LaserDetuningSampler::default(); crate::laser::COOLING_BEAM_LIMIT],
            })
            .build();

        let mut system = CalculateLaserDetuningSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserDetuningSamplers>();

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .contents[0]
                .detuning_sigma_plus,
            -10.0e6 - 10.0e6,
            1e-2_f64
        );

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .contents[0]
                .detuning_sigma_minus,
            -10.0e6 + 10.0e6,
            1e-2_f64
        );
        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .contents[0]
                .detuning_pi,
            -10.0e6,
            1e-2_f64
        );
    }
}
