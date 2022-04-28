//! Calculation of the steady-state twolevel populations

use crate::integrator::BatchSize;

use super::{rate::RateCoefficients, sampler_masks::CoolingLaserSamplerMasks};
use serde::{Deserialize, Serialize};
use bevy::{prelude::*, tasks::ComputeTaskPool};
use std::{fmt, marker::PhantomData};

use super::transition::{TransitionComponent};

/// Represents the steady-state population density of the excited state and ground state for a given atomic transition.
#[derive(Deserialize, Serialize, Clone, Component)]
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

/// Calculates the TwoLevelPopulation from the natural linewidth and the `RateCoefficients`
pub fn calculate_two_level_population<const N: usize, T : TransitionComponent>(
    mut atom_query: Query<(&mut TwoLevelPopulation<T>, &CoolingLaserSamplerMasks<N>, &RateCoefficients<T,N>), With<T>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>
) {
    atom_query.par_for_each_mut(&task_pool, batch_size.0,
        |(mut twolevel, mask, rates)| {
            let mut sum_rates: f64 = 0.;

            for count in 0..rates.contents.len() {
                if mask.contents[count].filled {
                    sum_rates += rates.contents[count].rate;
                }
            }
            twolevel.excited = sum_rates / (T::gamma() + 2. * sum_rates);
            twolevel.calculate_ground_state();
        }
    );
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::{laser::DEFAULT_BEAM_LIMIT, laser_cooling::sampler_masks::LaserSamplerMask, species::{Strontium88_461, Rubidium87_780D2}, laser_cooling::{rate::RateCoefficient, transition::AtomicTransition}};
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;

    #[test]
    fn test_calculate_twolevel_population_system() {
        let mut test_world = World::new();
        test_world.register::<RateCoefficients<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<CoolingLaserSamplerMasks<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TwoLevelPopulation<Strontium88_461>>();
        test_world.register::<Strontium88_461>();

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers =
            [LaserSamplerMask { filled: false }; DEFAULT_BEAM_LIMIT];
        active_lasers[0] = LaserSamplerMask { filled: true };
        active_lasers[1] = LaserSamplerMask { filled: true };

        let mut rc = RateCoefficient::<Strontium88_461>::default();
        rc.rate = 1_000_000.0;

        let atom1 = test_world
            .create_entity()
            .with(RateCoefficients  {
                contents: [rc; DEFAULT_BEAM_LIMIT],
            })
            .with(Strontium88_461)
            .with(CoolingLaserSamplerMasks {
                contents: active_lasers,
            })
            .with(TwoLevelPopulation::<Strontium88_461>::default())
            .build();

        let mut system = CalculateTwoLevelPopulationSystem::<Strontium88_461, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TwoLevelPopulation<Strontium88_461>>();

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
            sum_rates / (Strontium88_461::gamma() + 2.0 * sum_rates),
            1e-5_f64
        );
    }

    #[test]
    fn test_popn_high_intensity_limit() {
        let mut test_world = World::new();
        test_world.register::<RateCoefficients<Rubidium87_780D2, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Rubidium87_780D2>();
        test_world.register::<CoolingLaserSamplerMasks<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<TwoLevelPopulation<Rubidium87_780D2>>();

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers = [LaserSamplerMask { filled: false };
            crate::laser::DEFAULT_BEAM_LIMIT];
        active_lasers[0] = LaserSamplerMask { filled: true };

        let mut rc = RateCoefficient::<Rubidium87_780D2>::default();
        rc.rate = 1.0e9;

        let atom1 = test_world
            .create_entity()
            .with(RateCoefficients {
                contents: [rc; DEFAULT_BEAM_LIMIT],
            })
            .with(Rubidium87_780D2)
            .with(CoolingLaserSamplerMasks {
                contents: active_lasers,
            })
            .with(TwoLevelPopulation::<Rubidium87_780D2>::default())
            .build();

        let mut system = CalculateTwoLevelPopulationSystem::<Rubidium87_780D2, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TwoLevelPopulation<Rubidium87_780D2>>();

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
