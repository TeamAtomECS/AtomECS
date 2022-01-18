//! Calculation of the steady-state twolevel populations

extern crate rayon;

use crate::laser::sampler::CoolingLaserSamplerMasks;
use crate::laser_cooling::rate::RateCoefficients;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::{fmt, marker::PhantomData};

use super::transition::{TransitionComponent};

/// Represents the steady-state population density of the excited state and ground state for a given atomic transition.
#[derive(Deserialize, Serialize, Clone)]
pub struct TwoLevelPopulation<T> where T : TransitionComponent {
    /// steady-state population density of the ground state, a number in [0,1]
    pub ground: f64,
    /// steady-state population density of the excited state, a number in [0,1]
    pub excited: f64,
    marker: PhantomData<T>,
}

impl<T> fmt::Display for TwoLevelPopulation<T> where T : TransitionComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g:{},e:{}", self.ground, self.excited)
    }
}

impl<T> Default for TwoLevelPopulation<T> where T : TransitionComponent {
    fn default() -> Self {
        TwoLevelPopulation {
            /// steady-state population density of the ground state, a number in [0,1]
            ground: f64::NAN,
            /// steady-state population density of the excited state, a number in [0,1]
            excited: f64::NAN,
            marker: PhantomData
        }
    }
}

impl<T> TwoLevelPopulation<T> where T : TransitionComponent {
    /// Calculate the ground state population from excited state population
    pub fn calculate_ground_state(&mut self) {
        self.ground = 1. - self.excited;
    }
    /// Calculate the excited state population from ground state population
    pub fn calculate_excited_state(&mut self) {
        self.excited = 1. - self.ground;
    }
}

impl<T> Component for TwoLevelPopulation<T> where T : TransitionComponent + 'static {
    type Storage = VecStorage<Self>;
}

/// Calculates the TwoLevelPopulation from the natural linewidth and the `RateCoefficients`
#[derive(Default)]
pub struct CalculateTwoLevelPopulationSystem<T, const N: usize>(PhantomData<T>) where T: TransitionComponent;

impl<'a, T, const N: usize> System<'a> for CalculateTwoLevelPopulationSystem<T, N> where T: TransitionComponent {
    type SystemData = (
        ReadStorage<'a, T>,
        ReadStorage<'a, RateCoefficients<T, N>>,
        ReadStorage<'a, CoolingLaserSamplerMasks<N>>,
        WriteStorage<'a, TwoLevelPopulation<T>>,
    );

    fn run(
        &mut self,
        (transition, rate_coefficients, masks, mut twolevel_population): Self::SystemData,
    ) {
        use rayon::prelude::*;

        (
            &transition,
            &rate_coefficients,
            &masks,
            &mut twolevel_population,
        )
            .par_join()
            .for_each(|(_transition, rates, mask, twolevel)| {
                let mut sum_rates: f64 = 0.;

                for count in 0..rates.contents.len() {
                    if mask.contents[count].filled {
                        sum_rates += rates.contents[count].rate;
                    }
                }
                twolevel.excited = sum_rates / (T::gamma() + 2. * sum_rates);
                twolevel.calculate_ground_state();
            });
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::laser::DEFAULT_BEAM_LIMIT;
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;

    #[test]
    fn test_calculate_twolevel_population_system() {
        let mut test_world = World::new();
        test_world.register::<RateCoefficients<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<CoolingLaserSamplerMasks<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TwoLevelPopulation>();

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers =
            [crate::laser::sampler::LaserSamplerMask { filled: false }; DEFAULT_BEAM_LIMIT];
        active_lasers[0] = crate::laser::sampler::LaserSamplerMask { filled: true };
        active_lasers[1] = crate::laser::sampler::LaserSamplerMask { filled: true };

        let atom1 = test_world
            .create_entity()
            .with(RateCoefficients {
                contents: [crate::laser_cooling::rate::RateCoefficient { rate: 1_000_000.0 };
                    crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(TransitionComponent::strontium())
            .with(CoolingLaserSamplerMasks {
                contents: active_lasers,
            })
            .with(TwoLevelPopulation::default())
            .build();

        let mut system = CalculateTwoLevelPopulationSystem::<{ DEFAULT_BEAM_LIMIT }>;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TwoLevelPopulation>();

        let mut sum_rates = 0.0;

        for active_laser in active_lasers.iter().take(crate::laser::DEFAULT_BEAM_LIMIT) {
            if active_laser.filled {
                sum_rates += 1_000_000.0;
            }
        }

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .excited,
            sum_rates / (TransitionComponent::strontium().gamma() + 2.0 * sum_rates),
            1e-5_f64
        );
    }

    #[test]
    fn test_popn_high_intensity_limit() {
        let mut test_world = World::new();
        test_world.register::<RateCoefficients<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TransitionComponent>();
        test_world.register::<CoolingLaserSamplerMasks<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TwoLevelPopulation>();

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers = [crate::laser::sampler::LaserSamplerMask { filled: false };
            crate::laser::DEFAULT_BEAM_LIMIT];
        active_lasers[0] = crate::laser::sampler::LaserSamplerMask { filled: true };

        let atom1 = test_world
            .create_entity()
            .with(RateCoefficients {
                contents: [crate::laser_cooling::rate::RateCoefficient { rate: 1.0e9 };
                    crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(TransitionComponent::rubidium())
            .with(CoolingLaserSamplerMasks {
                contents: active_lasers,
            })
            .with(TwoLevelPopulation::default())
            .build();

        let mut system = CalculateTwoLevelPopulationSystem::<{ DEFAULT_BEAM_LIMIT }>;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TwoLevelPopulation>();

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .excited,
            0.5,
            0.01
        );
    }
}
