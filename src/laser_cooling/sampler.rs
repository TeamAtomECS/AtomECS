//! Calculation of the total detuning for specific atoms and CoolingLight entities

use super::CoolingLight;
use super::transition::TransitionComponent;
use crate::constant;
use crate::integrator::BatchSize;
use crate::laser::index::LaserIndex;
use crate::laser_cooling::doppler::DopplerShiftSamplers;
use super::zeeman::ZeemanShiftSampler;
use bevy::tasks::ComputeTaskPool;
use bevy::prelude::*;
use std::f64;
use std::marker::PhantomData;
extern crate nalgebra;

const LASER_CACHE_SIZE: usize = 16;

/// Represents total detuning of the atom's transition with respect to each beam
#[derive(Clone, Copy)]
pub struct LaserDetuningSampler<T> where T : TransitionComponent {
    /// Laser detuning of the sigma plus transition with respect to laser beam, in SI units of rad/s
    pub detuning_sigma_plus: f64,
    /// Laser detuning of the sigma minus transition with respect to laser beam, in SI units of rad/s
    pub detuning_sigma_minus: f64,
    /// Laser detuning of the pi transition with respect to laser beam, in SI units of rad/s
    pub detuning_pi: f64,
    phantom: PhantomData<T>
}

impl<T> Default for LaserDetuningSampler<T> where T : TransitionComponent {
    fn default() -> Self {
        LaserDetuningSampler {
            detuning_sigma_plus: f64::NAN,
            detuning_sigma_minus: f64::NAN,
            detuning_pi: f64::NAN,
            phantom: PhantomData
        }
    }
}

/// Component that holds a vector of `LaserDetuningSampler`
#[derive(Component)]
pub struct LaserDetuningSamplers<T, const N: usize> where T : TransitionComponent {
    /// List of `LaserDetuningSampler`s
    pub contents: [LaserDetuningSampler<T>; N],
}

/// Calculates the total laser detuning for each atom with respect to each [CoolingLight].
pub fn calculate_laser_detuning<const N: usize, T : TransitionComponent>(
    laser_query: Query<(&LaserIndex, &CoolingLight)>,
    mut atom_query: Query<(&mut LaserDetuningSamplers<T,N>, &DopplerShiftSamplers<N>, &ZeemanShiftSampler<T>), With<T>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>
) {
    // There are typically only a small number of lasers in a simulation.
    // For a speedup, cache the required components into thread memory,
    // so they can be distributed to parallel workers during the atom loop.
    type CachedLaser = (LaserIndex, CoolingLight);
    let mut laser_cache: Vec<CachedLaser> = Vec::new();
    for (index, cooling) in laser_query.iter() {
            laser_cache.push((*index, *cooling));
    }

    // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
    for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
        let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
        let slice = &laser_cache[base_index..max_index];
        let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
        laser_array[..max_index].copy_from_slice(slice);
        let number_in_iteration = slice.len();

        atom_query.par_for_each_mut(&task_pool, batch_size.0,
            |(mut detuning_sampler, doppler_samplers, zeeman_sampler)| {
                for (index, cooling) in laser_array.iter().take(number_in_iteration) {
                    let without_zeeman = 2.0
                        * constant::PI
                        * (constant::C / cooling.wavelength - T::frequency())
                        - doppler_samplers.contents[index.index].doppler_shift;

                    detuning_sampler.contents[index.index].detuning_sigma_plus =
                        without_zeeman - zeeman_sampler.sigma_plus;
                    detuning_sampler.contents[index.index].detuning_sigma_minus =
                        without_zeeman - zeeman_sampler.sigma_minus;
                    detuning_sampler.contents[index.index].detuning_pi =
                        without_zeeman - zeeman_sampler.sigma_pi;
                }
            },
        );
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{laser::DEFAULT_BEAM_LIMIT, species::Strontium88_461, laser_cooling::{transition::AtomicTransition, doppler::DopplerShiftSampler}};

    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;

    #[test]
    fn test_calculate_laser_detuning_system() {
        let mut test_world = World::new();
        test_world.register::<CoolingLight>();
        test_world.register::<LaserIndex>();
        test_world.register::<DopplerShiftSamplers<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<LaserDetuningSamplers<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Strontium88_461>();
        test_world.register::<ZeemanShiftSampler<Strontium88_461>>();

        let wavelength = constant::C / Strontium88_461::frequency();
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

        let mut zss = ZeemanShiftSampler::<Strontium88_461>::default();
        zss.sigma_pi = 0.0;
        zss.sigma_plus = 10.0e6;
        zss.sigma_minus = -10.0e6;

        let atom1 = test_world
            .create_entity()
            .with(DopplerShiftSamplers {
                contents: [DopplerShiftSampler {
                    doppler_shift: 10.0e6, //rad/s
                }; DEFAULT_BEAM_LIMIT],
            })
            .with(Strontium88_461)
            .with(zss)
            .with(LaserDetuningSamplers::<Strontium88_461, DEFAULT_BEAM_LIMIT> {
                contents: [LaserDetuningSampler::default(); DEFAULT_BEAM_LIMIT],
            })
            .build();

        let mut system = CalculateLaserDetuningSystem::<Strontium88_461, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage =
            test_world.read_storage::<LaserDetuningSamplers<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();

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
