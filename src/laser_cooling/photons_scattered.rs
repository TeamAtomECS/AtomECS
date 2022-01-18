//! Calculation of scattering events of photons with atoms

extern crate rayon;

use rand;
use rand_distr::{Distribution, Poisson};

use crate::{integrator::Timestep};
use crate::laser::sampler::CoolingLaserSamplerMasks;
use crate::laser_cooling::rate::RateCoefficients;
use crate::laser_cooling::twolevel::TwoLevelPopulation;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::fmt;
use std::marker::PhantomData;

use super::transition::{TransitionComponent};

/// Holds the total number of photons that the atom is expected to scatter
/// in the current simulation step from all beams.
///
/// This is an early estimation used to determine the more precise `ExpectedPhotonsScattered`
/// afterwards.
#[derive(Clone, Copy, Serialize)]
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

impl<T> Component for TotalPhotonsScattered<T> where T : TransitionComponent + 'static {
    type Storage = VecStorage<Self>;
}

/// Calcutates the total number of photons scattered in one iteration step
///
/// This can be calculated by: Timestep * TwolevelPopulation * Linewidth
#[derive(Default)]
pub struct CalculateMeanTotalPhotonsScatteredSystem<T>(PhantomData<T>) where T : TransitionComponent;
impl<'a, T> System<'a> for CalculateMeanTotalPhotonsScatteredSystem<T> 
where T: TransitionComponent {
    type SystemData = (
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, T>,
        ReadStorage<'a, TwoLevelPopulation<T>>,
        WriteStorage<'a, TotalPhotonsScattered<T>>,
    );

    fn run(
        &mut self,
        (timestep, transition, twolevel_population, mut total_photons_scattered): Self::SystemData,
    ) {
        use rayon::prelude::*;

        (
            &transition,
            &twolevel_population,
            &mut total_photons_scattered,
        )
            .par_join()
            .for_each(|(_atominfo, twolevel, total)| {
                total.total = timestep.delta * T::gamma() * twolevel.excited;
            });
    }
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
#[derive(Deserialize, Serialize, Clone)]
pub struct ExpectedPhotonsScatteredVector<T, const N: usize> where T : TransitionComponent {
    #[serde(with = "serde_arrays")]
    pub contents: [ExpectedPhotonsScattered<T>; N],
}

impl<T, const N: usize> Component for ExpectedPhotonsScatteredVector<T, N> where T : TransitionComponent {
    type Storage = VecStorage<Self>;
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

/// This system initialises all ´ExpectedPhotonsScatteredVector´ to a NAN value.
///
/// It also ensures that the size of the ´ExpectedPhotonsScatteredVector´ components match the number of CoolingLight entities in the world.
#[derive(Default)]
pub struct InitialiseExpectedPhotonsScatteredVectorSystem<T, const N: usize>(PhantomData<T>) where T : TransitionComponent;
impl<'a, T, const N: usize> System<'a> for InitialiseExpectedPhotonsScatteredVectorSystem<T, N> where T : TransitionComponent {
    type SystemData = (WriteStorage<'a, ExpectedPhotonsScatteredVector<T, N>>,);
    fn run(&mut self, (mut expected_photons,): Self::SystemData) {
        use rayon::prelude::*;

        (&mut expected_photons).par_join().for_each(|mut expected| {
            expected.contents = [ExpectedPhotonsScattered::default(); N];
        });
    }
}

/// Calculates the expected mean number of Photons scattered by each laser in one iteration step
///
/// It is required that the `TotalPhotonsScattered` is already updated since this System divides
/// them between the CoolingLight entities.
#[derive(Default)]
pub struct CalculateExpectedPhotonsScatteredSystem<T, const N: usize>(PhantomData<T>) where T : TransitionComponent;
impl<'a, T, const N: usize> System<'a> for CalculateExpectedPhotonsScatteredSystem<T, N> where T : TransitionComponent {
    type SystemData = (
        ReadStorage<'a, RateCoefficients<T, N>>,
        ReadStorage<'a, TotalPhotonsScattered<T>>,
        ReadStorage<'a, CoolingLaserSamplerMasks<N>>,
        WriteStorage<'a, ExpectedPhotonsScatteredVector<T, N>>,
    );

    fn run(
        &mut self,
        (rate_coefficients, total_photons_scattered, masks, mut expected_photons_vector): Self::SystemData,
    ) {
        use rayon::prelude::*;

        (
            &rate_coefficients,
            &total_photons_scattered,
            &masks,
            &mut expected_photons_vector,
        )
            .par_join()
            .for_each(|(rates, total, mask, expected)| {
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
            });
    }
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
#[derive(Deserialize, Serialize, Clone)]
pub struct ActualPhotonsScatteredVector<T, const N: usize> where T : TransitionComponent {
    #[serde(with = "serde_arrays")]
    pub contents: [ActualPhotonsScattered<T>; N],
}

impl<T, const N: usize> ActualPhotonsScatteredVector<T, N> where T : TransitionComponent{
    /// Calculate the sum of all entries
    pub fn calculate_total_scattered(&self) -> u64 {
        let mut sum: f64 = 0.0;
        // for i in 0..self.contents.len() {
        //     sum += self.contents[i].scattered;
        // }
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
impl<T, const N: usize> Component for ActualPhotonsScatteredVector<T, N> where T : TransitionComponent + 'static {
    type Storage = VecStorage<Self>;
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
#[derive(Default)]
pub struct CalculateActualPhotonsScatteredSystem<T, const N: usize>(PhantomData<T>) where T : TransitionComponent;

impl<'a, T, const N: usize> System<'a> for CalculateActualPhotonsScatteredSystem<T, N> where T : TransitionComponent {
    type SystemData = (
        Option<Read<'a, ScatteringFluctuationsOption>>,
        ReadStorage<'a, ExpectedPhotonsScatteredVector<T, N>>,
        WriteStorage<'a, ActualPhotonsScatteredVector<T, N>>,
    );

    fn run(
        &mut self,
        (fluctuations_option, expected_photons_vector, mut actual_photons_vector): Self::SystemData,
    ) {
        use rayon::prelude::*;

        match fluctuations_option {
            None => {
                (&expected_photons_vector, &mut actual_photons_vector)
                    .par_join()
                    .for_each(|(expected, actual)| {
                        for index in 0..expected.contents.len() {
                            actual.contents[index].scattered = expected.contents[index].scattered;
                        }
                    });
            }
            Some(rand_option) => match *rand_option {
                ScatteringFluctuationsOption::Off => {
                    (&expected_photons_vector, &mut actual_photons_vector)
                        .par_join()
                        .for_each(|(expected, actual)| {
                            for index in 0..expected.contents.len() {
                                actual.contents[index].scattered =
                                    expected.contents[index].scattered;
                            }
                        });
                }
                ScatteringFluctuationsOption::On => {
                    (&expected_photons_vector, &mut actual_photons_vector)
                        .par_join()
                        .for_each(|(expected, actual)| {
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
                        });
                }
            },
        }
    }
}

#[cfg(test)]
pub mod tests {

    use crate::laser::DEFAULT_BEAM_LIMIT;

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

        test_world.register::<TwoLevelPopulation>();
        test_world.register::<AtomicTransition>();
        test_world.register::<TotalPhotonsScattered>();
        test_world.insert(Timestep { delta: time_delta });

        let atom1 = test_world
            .create_entity()
            .with(TotalPhotonsScattered::default())
            .with(AtomicTransition::strontium())
            .with(TwoLevelPopulation {
                ground: 0.7,
                excited: 0.3,
            })
            .build();

        let mut system = CalculateMeanTotalPhotonsScatteredSystem;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TotalPhotonsScattered>();

        let scattered = AtomicTransition::strontium().gamma() * 0.3 * time_delta;

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

        test_world.register::<RateCoefficients<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<CoolingLaserSamplerMasks<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TotalPhotonsScattered>();
        test_world.register::<ExpectedPhotonsScatteredVector<{ DEFAULT_BEAM_LIMIT }>>();

        //We assume 16 beams with equal `RateCoefficient`s for this test

        let atom1 = test_world
            .create_entity()
            .with(TotalPhotonsScattered { total: 8.0 })
            .with(CoolingLaserSamplerMasks {
                contents: [crate::laser::sampler::LaserSamplerMask { filled: true };
                    crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(RateCoefficients {
                contents: [crate::laser_cooling::rate::RateCoefficient { rate: 1_000_000.0 };
                    crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(ExpectedPhotonsScatteredVector {
                contents: [ExpectedPhotonsScattered::default(); crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .build();
        let mut system = CalculateExpectedPhotonsScatteredSystem::<{ DEFAULT_BEAM_LIMIT }>;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage =
            test_world.read_storage::<ExpectedPhotonsScatteredVector<{ DEFAULT_BEAM_LIMIT }>>();

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
