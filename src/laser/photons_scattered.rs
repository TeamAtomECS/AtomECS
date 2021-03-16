//! Calculation of scattering events of photons with atoms

extern crate rayon;
extern crate specs;

extern crate rand;
use rand::distributions::{Distribution, Poisson};
use specs::Read;

use crate::atom::AtomicTransition;
use crate::integrator::Timestep;
use crate::laser::rate::RateCoefficients;
use crate::laser::sampler::LaserSamplerMasks;
use crate::laser::twolevel::TwoLevelPopulation;
use serde::{Deserialize, Serialize};
use specs::{Component, ReadExpect, ReadStorage, System, VecStorage, WriteStorage};
use std::fmt;
/// Holds the total number of photons that the atom is expected to scatter
/// in the current simulation step from all beams.
///
/// This is an early estimation used to determine the more precise `ExpectedPhotonsScattered`
/// afterwards.
#[derive(Clone)]
pub struct TotalPhotonsScattered {
    /// Number of photons scattered from all beams
    pub total: f64,
}

impl Default for TotalPhotonsScattered {
    fn default() -> Self {
        TotalPhotonsScattered {
            /// Number of photons scattered from all beams
            total: f64::NAN,
        }
    }
}

impl Component for TotalPhotonsScattered {
    type Storage = VecStorage<Self>;
}

/// Calcutates the total number of photons scattered in one iteration step
///
/// This can be calculated by: Timestep * TwolevelPopulation * Linewidth
pub struct CalculateMeanTotalPhotonsScatteredSystem;
impl<'a> System<'a> for CalculateMeanTotalPhotonsScatteredSystem {
    type SystemData = (
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, TwoLevelPopulation>,
        WriteStorage<'a, TotalPhotonsScattered>,
    );

    fn run(
        &mut self,
        (timestep, atomic_transition, twolevel_population, mut total_photons_scattered): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (
            &atomic_transition,
            &twolevel_population,
            &mut total_photons_scattered,
        )
            .par_join()
            .for_each(|(atominfo, twolevel, total)| {
                // DEFINITELY CHECK the 2pi!!!
                total.total = timestep.delta * atominfo.linewidth * twolevel.excited;
            });
    }
}

/// The number of photons scattered by the atom from a single, specific beam
#[derive(Clone, Copy)]
pub struct ExpectedPhotonsScattered {
    ///photons scattered by the atom from a specific beam
    scattered: f64,
}

impl Default for ExpectedPhotonsScattered {
    fn default() -> Self {
        ExpectedPhotonsScattered {
            ///photons scattered by the atom from a specific beam
            scattered: f64::NAN,
        }
    }
}

/// The List that holds an `ExpectedPhotonsScattered` for each laser
pub struct ExpectedPhotonsScatteredVector {
    pub contents: [ExpectedPhotonsScattered; crate::laser::COOLING_BEAM_LIMIT],
}

impl Component for ExpectedPhotonsScatteredVector {
    type Storage = VecStorage<Self>;
}

/// This system initialises all ´ExpectedPhotonsScatteredVector´ to a NAN value.
///
/// It also ensures that the size of the ´ExpectedPhotonsScatteredVector´ components match the number of CoolingLight entities in the world.
pub struct InitialiseExpectedPhotonsScatteredVectorSystem;
impl<'a> System<'a> for InitialiseExpectedPhotonsScatteredVectorSystem {
    type SystemData = (WriteStorage<'a, ExpectedPhotonsScatteredVector>,);
    fn run(&mut self, (mut expected_photons,): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut expected_photons).par_join().for_each(|mut expected| {
            expected.contents =
                [ExpectedPhotonsScattered::default(); crate::laser::COOLING_BEAM_LIMIT];
        });
    }
}

/// Calculates the expected mean number of Photons scattered by each laser in one iteration step
///
/// It is required that the `TotalPhotonsScattered` is already updated since this System divides
/// them between the CoolingLight entities.
pub struct CalculateExpectedPhotonsScatteredSystem;
impl<'a> System<'a> for CalculateExpectedPhotonsScatteredSystem {
    type SystemData = (
        ReadStorage<'a, RateCoefficients>,
        ReadStorage<'a, TotalPhotonsScattered>,
        ReadStorage<'a, LaserSamplerMasks>,
        WriteStorage<'a, ExpectedPhotonsScatteredVector>,
    );

    fn run(
        &mut self,
        (rate_coefficients, total_photons_scattered, masks, mut expected_photons_vector): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

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
                        sum_rates = sum_rates + rates.contents[index].rate;
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
pub struct ActualPhotonsScattered {
    ///  number of photons actually scattered by the atom from a specific beam
    pub scattered: f64,
}

impl Default for ActualPhotonsScattered {
    fn default() -> Self {
        ActualPhotonsScattered {
            ///  number of photons actually scattered by the atom from a specific beam
            scattered: 0.0,
        }
    }
}

/// The ist that holds an `ActualPhotonsScattered` for each CoolingLight entity
#[derive(Deserialize, Serialize, Clone)]
pub struct ActualPhotonsScatteredVector {
    pub contents: [ActualPhotonsScattered; crate::laser::COOLING_BEAM_LIMIT],
}

impl ActualPhotonsScatteredVector {
    /// Calculate the sum of all entries
    pub fn calculate_total_scattered(&self) -> u64 {
        let mut sum: f64 = 0.0;
        for i in 0..self.contents.len() {
            sum = sum + self.contents[i].scattered;
        }
        sum as u64
    }
}

impl fmt::Display for ActualPhotonsScatteredVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = f.write_str("");
        for aps in &self.contents {
            result = f.write_fmt(format_args!("{},", aps.scattered));
        }
        result
        //f.debug_list().entries(self.contents.iter()).finish()
    }
}

impl Component for ActualPhotonsScatteredVector {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `ActualPhotonsScatteredVector` to a NAN value.
///
/// It also ensures that the size of the `ActualPhotonsScatteredVector` components match the number of CoolingLight entities in the world.
pub struct InitialiseActualPhotonsScatteredVectorSystem;
impl<'a> System<'a> for InitialiseActualPhotonsScatteredVectorSystem {
    type SystemData = (WriteStorage<'a, ActualPhotonsScatteredVector>,);
    fn run(&mut self, (mut actual_photons,): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut actual_photons).par_join().for_each(|mut actual| {
            actual.contents = [ActualPhotonsScattered::default(); crate::laser::COOLING_BEAM_LIMIT];
        });
    }
}

/// If this is added as a ressource, the number of actual photons will be drawn from a poisson distribution.
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
pub struct CalculateActualPhotonsScatteredSystem;
impl<'a> System<'a> for CalculateActualPhotonsScatteredSystem {
    type SystemData = (
        Option<Read<'a, ScatteringFluctuationsOption>>,
        ReadStorage<'a, ExpectedPhotonsScatteredVector>,
        WriteStorage<'a, ActualPhotonsScatteredVector>,
    );

    fn run(
        &mut self,
        (fluctuations_option, expected_photons_vector, mut actual_photons_vector): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

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
                                        let poisson = Poisson::new(lambda);
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
        test_world.add_resource(Timestep { delta: time_delta });

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
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TotalPhotonsScattered>();

        let scattered = AtomicTransition::strontium().linewidth * 0.3 * time_delta;

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

        test_world.register::<RateCoefficients>();
        test_world.register::<LaserSamplerMasks>();
        test_world.register::<TotalPhotonsScattered>();
        test_world.register::<ExpectedPhotonsScatteredVector>();

        //We assume 16 beams with equal `RateCoefficient`s for this test

        let atom1 = test_world
            .create_entity()
            .with(TotalPhotonsScattered { total: 8.0 })
            .with(LaserSamplerMasks {
                contents: [crate::laser::sampler::LaserSamplerMask { filled: true };
                    crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(RateCoefficients {
                contents: [crate::laser::rate::RateCoefficient { rate: 1_000_000.0 };
                    crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(ExpectedPhotonsScatteredVector {
                contents: [ExpectedPhotonsScattered::default(); crate::laser::COOLING_BEAM_LIMIT],
            })
            .build();
        let mut system = CalculateExpectedPhotonsScatteredSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<ExpectedPhotonsScatteredVector>();

        let scattered = 8.0 / crate::laser::COOLING_BEAM_LIMIT as f64;

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
