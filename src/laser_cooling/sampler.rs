//! Calculation of the total detuning for specific atoms and CoolingLight entities

use super::CoolingLight;
use crate::atom::AtomicTransition;
use crate::constant;
use crate::laser::index::LaserIndex;
use crate::laser_cooling::doppler::DopplerShiftSamplers;
use crate::magnetic::zeeman::ZeemanShiftSampler;
use specs::prelude::*;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};
use std::f64;
extern crate nalgebra;

const LASER_CACHE_SIZE: usize = 16;

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
    pub contents: [LaserDetuningSampler; crate::laser::BEAM_LIMIT],
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

        (&mut samplers).par_join().for_each(|mut sampler| {
            sampler.contents = [LaserDetuningSampler::default(); crate::laser::BEAM_LIMIT];
        });
    }
}

/// This system calculates the total Laser Detuning for each atom with respect to
/// each CoolingLight entities.
pub struct CalculateLaserDetuningSystem;
impl<'a> System<'a> for CalculateLaserDetuningSystem {
    type SystemData = (
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, LaserIndex>,
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

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = (LaserIndex, CoolingLight);
        let laser_cache: Vec<CachedLaser> = (&indices, &cooling_light)
            .join()
            .map(|(index, cooling)| (*index, *cooling))
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
                        for (index, cooling) in laser_array.iter().take(number_in_iteration) {
                            let without_zeeman = 2.0
                                * constant::PI
                                * (constant::C / cooling.wavelength - atom_info.frequency)
                                - doppler_samplers.contents[index.index].doppler_shift;

                            detuning_sampler.contents[index.index].detuning_sigma_plus =
                                without_zeeman - zeeman_sampler.sigma_plus;
                            detuning_sampler.contents[index.index].detuning_sigma_minus =
                                without_zeeman - zeeman_sampler.sigma_minus;
                            detuning_sampler.contents[index.index].detuning_pi =
                                without_zeeman - zeeman_sampler.sigma_pi;
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
        test_world.register::<LaserIndex>();
        test_world.register::<DopplerShiftSamplers>();
        test_world.register::<LaserDetuningSamplers>();
        test_world.register::<AtomicTransition>();
        test_world.register::<ZeemanShiftSampler>();

        let wavelength = constant::C / AtomicTransition::strontium().frequency;
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength,
            })
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(DopplerShiftSamplers {
                contents: [crate::laser_cooling::doppler::DopplerShiftSampler {
                    doppler_shift: 10.0e6, //rad/s
                }; crate::laser::BEAM_LIMIT],
            })
            .with(AtomicTransition::strontium())
            .with(ZeemanShiftSampler {
                sigma_plus: 10.0e6,   //rad/s
                sigma_minus: -10.0e6, //rad/s
                sigma_pi: 0.0,        //rad/s
            })
            .with(LaserDetuningSamplers {
                contents: [LaserDetuningSampler::default(); crate::laser::BEAM_LIMIT],
            })
            .build();

        let mut system = CalculateLaserDetuningSystem;
        system.run_now(&test_world);
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
