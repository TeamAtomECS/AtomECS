//! Calculation of the steady-state twolevel populations

use crate::integrator::AtomECSBatchStrategy;

use super::{rate::RateCoefficients, sampler_masks::CoolingLaserSamplerMasks};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt, marker::PhantomData};

use super::transition::TransitionComponent;

/// Represents the steady-state population density of the excited state and ground state for a given atomic transition.
#[derive(Deserialize, Serialize, Clone, Component)]
pub struct TwoLevelPopulation<T>
where
    T: TransitionComponent,
{
    // steady-state population density of the ground state, a LASER_COUNT
    //  in [0,1]
    pub ground: f64,
    // steady-state population density of the excited state, a LASER_COUNT
    //  in [0,1]
    pub excited: f64,
    marker: PhantomData<T>,
}
impl<T> fmt::Display for TwoLevelPopulation<T>
where
    T: TransitionComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g:{},e:{}", self.ground, self.excited)
    }
}
impl<T> Default for TwoLevelPopulation<T>
where
    T: TransitionComponent,
{
    fn default() -> Self {
        TwoLevelPopulation {
            ground: f64::NAN,
            excited: f64::NAN,
            marker: PhantomData,
        }
    }
}
impl<T> TwoLevelPopulation<T>
where
    T: TransitionComponent,
{
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
pub fn calculate_two_level_population<const N: usize, T: TransitionComponent>(
    mut atom_query: Query<
        (
            &mut TwoLevelPopulation<T>,
            &CoolingLaserSamplerMasks<N>,
            &RateCoefficients<T, N>,
        ),
        With<T>,
    >,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut twolevel, mask, rates)| {
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

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::{
        laser_cooling::{
            rate::RateCoefficient, sampler_masks::CoolingLaserSamplerMask,
            transition::AtomicTransition,
        },
        species::Strontium88_461,
    };
    use assert_approx_eq::assert_approx_eq;

    const LASER_COUNT: usize = 4;

    #[test]
    fn test_calculate_twolevel_population_system() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());

        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers = [CoolingLaserSamplerMask { filled: false }; LASER_COUNT];
        active_lasers[0] = CoolingLaserSamplerMask { filled: true };
        active_lasers[1] = CoolingLaserSamplerMask { filled: true };

        let mut rc = RateCoefficient::<Strontium88_461>::default();
        rc.rate = 1_000_000.0;

        let atom1 = app
            .world_mut()
            .spawn(RateCoefficients {
                contents: [rc; LASER_COUNT],
            })
            .insert(Strontium88_461)
            .insert(CoolingLaserSamplerMasks {
                contents: active_lasers,
            })
            .insert(TwoLevelPopulation::<Strontium88_461>::default())
            .id();

        app.add_systems(Update, calculate_two_level_population::<LASER_COUNT, Strontium88_461>);
        app.update();

        let mut sum_rates = 0.0;

        for active_laser in active_lasers.iter().take(LASER_COUNT) {
            if active_laser.filled {
                sum_rates += 1_000_000.0;
            }
        }

        assert_approx_eq!(
            app.world()
                .entity(atom1)
                .get::<TwoLevelPopulation::<Strontium88_461>>()
                .expect("entity not found")
                .excited,
            sum_rates / (Strontium88_461::gamma() + 2.0 * sum_rates),
            1e-5_f64
        );
    }

    #[test]
    fn test_popn_high_intensity_limit() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());
        // this test runs with two lasers only and we have to tell this the mask
        let mut active_lasers = [CoolingLaserSamplerMask { filled: true }; LASER_COUNT];
        active_lasers[0] = CoolingLaserSamplerMask { filled: true };
        active_lasers[1] = CoolingLaserSamplerMask { filled: true };

        let mut rc = RateCoefficient::<Strontium88_461>::default();
        rc.rate = 1.0e10;

        let atom1 = app
            .world_mut()
            .spawn(RateCoefficients {
                contents: [rc; LASER_COUNT],
            })
            .insert(Strontium88_461)
            .insert(CoolingLaserSamplerMasks {
                contents: active_lasers,
            })
            .insert(TwoLevelPopulation::<Strontium88_461>::default())
            .id();

        app.add_systems(Update, calculate_two_level_population::<LASER_COUNT, Strontium88_461>);
        app.update();

        assert_approx_eq!(
            app.world()
                .entity(atom1)
                .get::<TwoLevelPopulation::<Strontium88_461>>()
                .expect("entity not found")
                .excited,
            0.5,
            0.01
        );
    }
}
