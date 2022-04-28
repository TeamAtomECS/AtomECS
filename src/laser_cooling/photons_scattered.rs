//! Calculation of scattering events of photons with atoms

use rand;
use rand_distr::{Distribution, Poisson};

use crate::{integrator::{Timestep, BatchSize}};
use super::sampler_masks::CoolingLaserSamplerMasks;
use crate::laser_cooling::rate::RateCoefficients;
use crate::laser_cooling::twolevel::TwoLevelPopulation;
use serde::{Deserialize, Serialize};
use bevy::{prelude::*, tasks::ComputeTaskPool};
use std::fmt;
use std::marker::PhantomData;

use super::transition::{TransitionComponent};

/// Holds the total number of photons that the atom is expected to scatter
/// in the current simulation step from all beams.
///
/// This is an early estimation used to determine the more precise `ExpectedPhotonsScattered`
/// afterwards.
#[derive(Clone, Copy, Serialize, Component)]
pub struct TotalPhotonsScattered<T> where T : TransitionComponent {
    /// Number of photons scattered from all beams
    pub total: f64,
    phantom: PhantomData<T>
}

impl<T> Default for TotalPhotonsScattered<T> where T : TransitionComponent {
    fn default() -> Self {
        TotalPhotonsScattered {
            /// Number of photons scattered from all beams
            total: f64::NAN,
            phantom: PhantomData
        }
    }
}

/// Calcutates the total number of photons scattered in one iteration step
///
/// This can be calculated by: Timestep * TwolevelPopulation * Linewidth
pub fn calculate_mean_total_photons_scattered<T : TransitionComponent>(
    mut query: Query<(&TwoLevelPopulation<T>, &mut TotalPhotonsScattered<T>), With<T>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
    timestep: Res<Timestep>
) {
    query.par_for_each_mut(
        &task_pool,
        batch_size.0,
        |(twolevel, mut total)| {
            total.total = timestep.delta * T::gamma() * twolevel.excited;
        }
    );
}

/// The number of photons scattered by the atom from a single, specific beam
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ExpectedPhotonsScattered<T> where T : TransitionComponent {
    ///photons scattered by the atom from a specific beam
    scattered: f64,
    phantom: PhantomData<T>
}
impl<T> Default for ExpectedPhotonsScattered<T> where T : TransitionComponent {
    fn default() -> Self {
        ExpectedPhotonsScattered {
            ///photons scattered by the atom from a specific beam
            scattered: f64::NAN,
            phantom: PhantomData
        }
    }
}

/// The List that holds an `ExpectedPhotonsScattered` for each laser
#[derive(Deserialize, Serialize, Clone, Component)]
pub struct ExpectedPhotonsScatteredVector<T, const N: usize> where T : TransitionComponent {
    #[serde(with = "serde_arrays")]
    pub contents: [ExpectedPhotonsScattered<T>; N],
}

impl<T, const N: usize> fmt::Display for ExpectedPhotonsScatteredVector<T, N> where T : TransitionComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = f.write_str("");
        for aps in &self.contents {
            result = f.write_fmt(format_args!("{},", aps.scattered));
        }
        result
    }
}

/// Calculates the expected mean number of Photons scattered by each laser in one iteration step
///
/// It is required that the `TotalPhotonsScattered` is already updated since this System divides
/// them between the CoolingLight entities.
pub fn calculate_expected_photons_scattered<const N: usize, T : TransitionComponent>(
    mut query: Query<(&mut ExpectedPhotonsScatteredVector<T,N>, &RateCoefficients<T,N>, &CoolingLaserSamplerMasks<N>, &TotalPhotonsScattered<T>)>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>
) {
    query.par_for_each_mut(&task_pool, batch_size.0, 
        |(mut expected, rates, mask, total)| {
            let mut sum_rates: f64 = 0.;

            for index in 0..rates.contents.len() {
                if mask.contents[index].filled {
                    sum_rates += rates.contents[index].rate;
                }
            }

            for index in 0..expected.contents.len() {
                if mask.contents[index].filled {
                    expected.contents[index].scattered =
                        rates.contents[index].rate / sum_rates * total.total;
                }
            }
        }
    );
}

/// The number of photons actually scattered by the atom from a single, specific beam
///
/// If `EnableScatteringFluctuations` is not activated, this will be the same as
/// `ExpectedPhotonsScattered`.
///
/// If `EnableScatteringFluctuations` is activated, this number represents the outcome
/// of a sampling process from a poisson distribution where the lambda parameter is
/// `ExpectedPhotonsScattered`. This adds an additional degree of randomness to
/// the simulation that helps to recreate the recoil limit.  
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct ActualPhotonsScattered<T> where T : TransitionComponent {
    ///  number of photons actually scattered by an atomic transition from a specific beam.
    pub scattered: f64,
    phantom: PhantomData<T>
}

impl<T> Default for ActualPhotonsScattered<T> where T : TransitionComponent {
    fn default() -> Self {
        ActualPhotonsScattered {
            ///  number of photons actually scattered by the atom from a specific beam
            scattered: 0.0,
            phantom: PhantomData
        }
    }
}

/// The ist that holds an `ActualPhotonsScattered` for each CoolingLight entity
#[derive(Deserialize, Serialize, Clone, Component)]
pub struct ActualPhotonsScatteredVector<T, const N: usize> where T : TransitionComponent {
    #[serde(with = "serde_arrays")]
    pub contents: [ActualPhotonsScattered<T>; N],
}
impl<T, const N: usize> ActualPhotonsScatteredVector<T, N> where T : TransitionComponent{
    /// Calculate the sum of all entries
    pub fn calculate_total_scattered(&self) -> u64 {
        let mut sum: f64 = 0.0;
        for item in &self.contents {
            sum += item.scattered;
        }
        sum as u64
    }
}
impl<T, const N: usize> fmt::Display for ActualPhotonsScatteredVector<T, N> where T : TransitionComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = f.write_str("");
        for aps in &self.contents {
            result = f.write_fmt(format_args!("{},", aps.scattered));
        }
        result
    }
}

/// If this is added as a resource, the number of actual photons will be drawn from a poisson distribution.
///
/// Otherwise, the entries of `ActualPhotonsScatteredVector` will be identical with those of
/// `ExpectedPhotonsScatteredVector`.
#[derive(Clone, Copy)]
pub enum ScatteringFluctuationsOption {
    Off,
    On,
}
impl Default for ScatteringFluctuationsOption {
    fn default() -> Self {
        ScatteringFluctuationsOption::On
    }
}

/// Calcutates the actual number of photons scattered by each CoolingLight entity in one iteration step
/// by drawing from a Poisson Distribution that has `ExpectedPhotonsScattered` as the lambda parameter.
pub fn calculate_actual_photons_scattered<const N: usize, T : TransitionComponent>(
    mut query: Query<(&ExpectedPhotonsScatteredVector<T,N>, &mut ActualPhotonsScatteredVector<T,N>)>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
    fluctuations: Res<ScatteringFluctuationsOption>
) {
    match fluctuations.as_ref() {
        ScatteringFluctuationsOption::Off => {
            query.par_for_each_mut(&task_pool, batch_size.0, 
                |(expected, mut actual)| {
                    for index in 0..expected.contents.len() {
                        actual.contents[index].scattered = expected.contents[index].scattered;
                    }
                }
            );
        }
        ScatteringFluctuationsOption::On => {
            query.par_for_each_mut(&task_pool, batch_size.0,
                |(expected, mut actual)| {
                    for index in 0..expected.contents.len() {
                        let lambda = expected.contents[index].scattered;
                        actual.contents[index].scattered =
                            if lambda <= 1.0e-5 || lambda.is_nan() {
                                0.0
                            } else {
                                let poisson = Poisson::new(lambda).unwrap();
                                let drawn_number = poisson.sample(&mut rand::thread_rng());
                                drawn_number as f64
                            }
                    }
                }
            );
        },
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{laser::{DEFAULT_BEAM_LIMIT}, species::Strontium88_461, laser_cooling::{rate::RateCoefficient, transition::AtomicTransition}};
    use crate::laser_cooling::sampler_masks::LaserSamplerMask;
    use super::*;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;

    /// Tests the correct implementation of the `CalculateMeanTotalPhotonsScatteredSystem`
    #[test]
    fn test_calculate_mean_total_photons_scattered_system() {
        let mut test_world = World::new();

        let time_delta = 1.0e-6;

        test_world.register::<TwoLevelPopulation<Strontium88_461>>();
        test_world.register::<Strontium88_461>();
        test_world.register::<TotalPhotonsScattered<Strontium88_461>>();
        test_world.insert(Timestep { delta: time_delta });

        let mut tlp = TwoLevelPopulation::<Strontium88_461>::default();
        tlp.ground = 0.7;
        tlp.excited = 0.3;

        let atom1 = test_world
            .create_entity()
            .with(TotalPhotonsScattered::<Strontium88_461>::default())
            .with(Strontium88_461)
            .with(tlp)
            .build();

        let mut system = CalculateMeanTotalPhotonsScatteredSystem::<Strontium88_461>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TotalPhotonsScattered<Strontium88_461>>();

        let scattered = Strontium88_461::gamma() * 0.3 * time_delta;

        assert_approx_eq!(
            sampler_storage.get(atom1).expect("entity not found").total,
            scattered,
            1e-5_f64
        );
    }

    /// Tests the correct implementation of the `CalculateExpectedPhotonsScatteredSystem`
    #[test]
    fn test_calculate_expected_photons_scattered_system() {
        let mut test_world = World::new();

        test_world.register::<RateCoefficients<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<CoolingLaserSamplerMasks<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TotalPhotonsScattered<Strontium88_461>>();
        test_world.register::<ExpectedPhotonsScatteredVector<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();

        //We assume 16 beams with equal `RateCoefficient`s for this test
        let mut rc = RateCoefficient::<Strontium88_461>::default();
        rc.rate = 1_000_000.0;
        let mut tps = TotalPhotonsScattered::<Strontium88_461>::default();
        tps.total = 8.0;

        let atom1 = test_world
            .create_entity()
            .with(tps)
            .with(CoolingLaserSamplerMasks {
                contents: [LaserSamplerMask { filled: true };
                    DEFAULT_BEAM_LIMIT],
            })
            .with(RateCoefficients {
                contents: [rc; DEFAULT_BEAM_LIMIT],
            })
            .with(ExpectedPhotonsScatteredVector {
                contents: [ExpectedPhotonsScattered::<Strontium88_461>::default(); crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .build();
        let mut system = CalculateExpectedPhotonsScatteredSystem::<Strontium88_461, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage =
            test_world.read_storage::<ExpectedPhotonsScatteredVector<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();

        let scattered = 8.0 / crate::laser::DEFAULT_BEAM_LIMIT as f64;

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .contents[12] //any entry between 0 and 15 should be the same
                .scattered,
            scattered,
            1e-5_f64
        );
    }
}
