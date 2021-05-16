//! Calculation of the steady-state twolevel populations

extern crate rayon;
extern crate specs;

use crate::atom::AtomicTransition;
use crate::laser::rate::RateCoefficients;
use crate::laser::sampler::LaserSamplerMasks;
use serde::{Deserialize, Serialize};
use specs::{Component, ReadStorage, System, VecStorage, WriteStorage};
use std::fmt;

/// Represents the steady-state population density of the excited state and ground state
#[derive(Deserialize, Serialize, Clone)]
pub struct TwoLevelPopulation {
    /// steady-state population density of the ground state, a number in [0,1]
    pub ground: f64,
    /// steady-state population density of the excited state, a number in [0,1]
    pub excited: f64,
}

impl fmt::Display for TwoLevelPopulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g:{},e:{}", self.ground, self.excited)
    }
}

impl Default for TwoLevelPopulation {
    fn default() -> Self {
        TwoLevelPopulation {
            /// steady-state population density of the ground state, a number in [0,1]
            ground: f64::NAN,
            /// steady-state population density of the excited state, a number in [0,1]
            excited: f64::NAN,
        }
    }
}

impl TwoLevelPopulation {
    /// Calculate the ground state population from excited state population
    pub fn calculate_ground_state(&mut self) {
        self.ground = 1. - self.excited;
    }
    /// Calculate the excited state population from ground state population
    pub fn calculate_excited_state(&mut self) {
        self.excited = 1. - self.ground;
    }
}

impl Component for TwoLevelPopulation {
    type Storage = VecStorage<Self>;
}

/// Calculates the TwoLevelPopulation from the natural linewidth and the `RateCoefficients`
pub struct CalculateTwoLevelPopulationSystem;
impl<'a> System<'a> for CalculateTwoLevelPopulationSystem {
    type SystemData = (
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, RateCoefficients>,
        ReadStorage<'a, LaserSamplerMasks>,
        WriteStorage<'a, TwoLevelPopulation>,
    );

    fn run(
        &mut self,
        (atomic_transition, rate_coefficients, masks, mut twolevel_population): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (
            &atomic_transition,
            &rate_coefficients,
            &masks,
            &mut twolevel_population,
        )
            .par_join()
            .for_each(|(atominfo, rates, mask, twolevel)| {
                let mut sum_rates: f64 = 0.;

                for count in 0..rates.contents.len() {
                    if mask.contents[count].filled {
                        sum_rates = sum_rates + rates.contents[count].rate;
                    }
                }
                twolevel.excited = sum_rates / (atominfo.gamma() + 2. * sum_rates);
                twolevel.calculate_ground_state();
            });
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
    fn test_calculate_twolevel_population_system() {
        let mut test_world = World::new();
        test_world.register::<RateCoefficients>();
        test_world.register::<AtomicTransition>();
        test_world.register::<LaserSamplerMasks>();
        test_world.register::<TwoLevelPopulation>();

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers = [crate::laser::sampler::LaserSamplerMask { filled: false };
            crate::laser::COOLING_BEAM_LIMIT];
        active_lasers[0] = crate::laser::sampler::LaserSamplerMask { filled: true };
        active_lasers[1] = crate::laser::sampler::LaserSamplerMask { filled: true };

        let atom1 = test_world
            .create_entity()
            .with(RateCoefficients {
                contents: [crate::laser::rate::RateCoefficient { rate: 1_000_000.0 };
                    crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(AtomicTransition::strontium())
            .with(LaserSamplerMasks {
                contents: active_lasers,
            })
            .with(TwoLevelPopulation::default())
            .build();

        let mut system = CalculateTwoLevelPopulationSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<TwoLevelPopulation>();

        let mut sum_rates = 0.0;

        for i in 0..crate::laser::COOLING_BEAM_LIMIT {
            if active_lasers[i].filled {
                sum_rates = sum_rates + 1_000_000.0;
            }
        }

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .excited,
            sum_rates / (AtomicTransition::strontium().gamma() + 2.0 * sum_rates),
            1e-5_f64
        );
    }

    #[test]
    fn test_popn_high_intensity_limit() {
        let mut test_world = World::new();
        test_world.register::<RateCoefficients>();
        test_world.register::<AtomicTransition>();
        test_world.register::<LaserSamplerMasks>();
        test_world.register::<TwoLevelPopulation>();

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers = [crate::laser::sampler::LaserSamplerMask { filled: false };
            crate::laser::COOLING_BEAM_LIMIT];
        active_lasers[0] = crate::laser::sampler::LaserSamplerMask { filled: true };

        let atom1 = test_world
            .create_entity()
            .with(RateCoefficients {
                contents: [crate::laser::rate::RateCoefficient { rate: 1.0e9 };
                    crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(AtomicTransition::rubidium())
            .with(LaserSamplerMasks {
                contents: active_lasers,
            })
            .with(TwoLevelPopulation::default())
            .build();

        let mut system = CalculateTwoLevelPopulationSystem;
        system.run_now(&test_world.res);
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
