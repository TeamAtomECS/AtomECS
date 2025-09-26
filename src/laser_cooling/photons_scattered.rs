//! Calculation of scattering events of photons with atoms

use rand;
use rand_distr::{Distribution, Poisson};

use super::sampler_masks::CoolingLaserSamplerMasks;
use crate::integrator::{AtomECSBatchStrategy, Timestep};
use crate::laser_cooling::rate::RateCoefficients;
use crate::laser_cooling::twolevel::TwoLevelPopulation;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

use super::transition::TransitionComponent;

/// Holds the total number of photons that the atom is expected to scatter
/// in the current simulation step from all beams.
///
/// This is an early estimation used to determine the more precise `ExpectedPhotonsScattered`
/// afterwards.
#[derive(Clone, Copy, Serialize, Component)]
pub struct TotalPhotonsScattered<T>
where
    T: TransitionComponent,
{
    /// Number of photons scattered from all beams
    pub total: f64,
    phantom: PhantomData<T>,
}

impl<T> Default for TotalPhotonsScattered<T>
where
    T: TransitionComponent,
{
    fn default() -> Self {
        TotalPhotonsScattered {
            // Number of photons scattered from all beams
            total: f64::NAN,
            phantom: PhantomData,
        }
    }
}

/// Calcutates the total number of photons scattered in one iteration step
///
/// This can be calculated by: Timestep * TwolevelPopulation * Linewidth
pub fn calculate_mean_total_photons_scattered<T: TransitionComponent>(
    mut query: Query<(&TwoLevelPopulation<T>, &mut TotalPhotonsScattered<T>), With<T>>,
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(twolevel, mut total)| {
            total.total = timestep.delta * T::gamma() * twolevel.excited;
        });
}

/// The number of photons scattered by the atom from a single, specific beam
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ExpectedPhotonsScattered<T>
where
    T: TransitionComponent,
{
    ///photons scattered by the atom from a specific beam
    scattered: f64,
    phantom: PhantomData<T>,
}
impl<T> Default for ExpectedPhotonsScattered<T>
where
    T: TransitionComponent,
{
    fn default() -> Self {
        ExpectedPhotonsScattered {
            // photons scattered by the atom from a specific beam
            scattered: f64::NAN,
            phantom: PhantomData,
        }
    }
}

/// The List that holds an `ExpectedPhotonsScattered` for each laser
#[derive(Deserialize, Serialize, Clone, Component)]
pub struct ExpectedPhotonsScatteredVector<T, const N: usize>
where
    T: TransitionComponent,
{
    #[serde(with = "serde_arrays")]
    pub contents: [ExpectedPhotonsScattered<T>; N],
}

impl<T, const N: usize> fmt::Display for ExpectedPhotonsScatteredVector<T, N>
where
    T: TransitionComponent,
{
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
pub fn calculate_expected_photons_scattered<const N: usize, T: TransitionComponent>(
    mut query: Query<(
        &mut ExpectedPhotonsScatteredVector<T, N>,
        &RateCoefficients<T, N>,
        &CoolingLaserSamplerMasks<N>,
        &TotalPhotonsScattered<T>,
    )>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut expected, rates, mask, total)| {
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
pub struct ActualPhotonsScattered<T>
where
    T: TransitionComponent,
{
    ///  number of photons actually scattered by an atomic transition from a specific beam.
    pub scattered: f64,
    phantom: PhantomData<T>,
}

impl<T> Default for ActualPhotonsScattered<T>
where
    T: TransitionComponent,
{
    fn default() -> Self {
        ActualPhotonsScattered {
            //  number of photons actually scattered by the atom from a specific beam
            scattered: 0.0,
            phantom: PhantomData,
        }
    }
}

/// The ist that holds an `ActualPhotonsScattered` for each CoolingLight entity
#[derive(Deserialize, Serialize, Clone, Component)]
pub struct ActualPhotonsScatteredVector<T, const N: usize>
where
    T: TransitionComponent,
{
    #[serde(with = "serde_arrays")]
    pub contents: [ActualPhotonsScattered<T>; N],
}
impl<T, const N: usize> ActualPhotonsScatteredVector<T, N>
where
    T: TransitionComponent,
{
    /// Calculate the sum of all entries
    pub fn calculate_total_scattered(&self) -> u64 {
        let mut sum: f64 = 0.0;
        for item in &self.contents {
            sum += item.scattered;
        }
        sum as u64
    }
}
impl<T, const N: usize> fmt::Display for ActualPhotonsScatteredVector<T, N>
where
    T: TransitionComponent,
{
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
#[derive(Clone, Copy, Resource, Default)]
pub enum ScatteringFluctuationsOption {
    Off,
    #[default]
    On,
}

/// Calcutates the actual number of photons scattered by each CoolingLight entity in one iteration step
/// by drawing from a Poisson Distribution that has `ExpectedPhotonsScattered` as the lambda parameter.
pub fn calculate_actual_photons_scattered<const N: usize, T: TransitionComponent>(
    mut query: Query<(
        &ExpectedPhotonsScatteredVector<T, N>,
        &mut ActualPhotonsScatteredVector<T, N>,
    )>,
    batch_strategy: Res<AtomECSBatchStrategy>,
    fluctuations: Res<ScatteringFluctuationsOption>,
) {
    match fluctuations.as_ref() {
        ScatteringFluctuationsOption::Off => {
            query
                .par_iter_mut()
                .batching_strategy(batch_strategy.0.clone())
                .for_each(|(expected, mut actual)| {
                    for index in 0..expected.contents.len() {
                        actual.contents[index].scattered = expected.contents[index].scattered;
                    }
                });
        }
        ScatteringFluctuationsOption::On => {
            query
                .par_iter_mut()
                .batching_strategy(batch_strategy.0.clone())
                .for_each(|(expected, mut actual)| {
                    for index in 0..expected.contents.len() {
                        let lambda = expected.contents[index].scattered;
                        actual.contents[index].scattered = if lambda <= 1.0e-5 || lambda.is_nan() {
                            0.0
                        } else {
                            let poisson = Poisson::new(lambda).unwrap();

                            poisson.sample(&mut rand::rng())
                        }
                    }
                });
        }
    }
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

    /// Tests the correct implementation of the `CalculateMeanTotalPhotonsScatteredSystem`
    #[test]
    fn test_calculate_mean_total_photons_scattered_system() {
        let mut app = App::new();
        let time_delta = 1.0e-6;
        app.insert_resource(AtomECSBatchStrategy::default());
        app.insert_resource(Timestep { delta: time_delta });

        let mut tlp = TwoLevelPopulation::<Strontium88_461>::default();
        tlp.ground = 0.7;
        tlp.excited = 0.3;

        let atom1 = app
            .world_mut()
            .spawn(TotalPhotonsScattered::<Strontium88_461>::default())
            .insert(Strontium88_461)
            .insert(tlp)
            .id();

        app.add_systems(Update, calculate_mean_total_photons_scattered::<Strontium88_461>);
        app.update();

        let scattered = Strontium88_461::gamma() * 0.3 * time_delta;

        assert_approx_eq!(
            app.world()
                .entity(atom1)
                .get::<TotalPhotonsScattered<Strontium88_461>>()
                .expect("entity not found")
                .total,
            scattered,
            1e-5_f64
        );
    }

    /// Tests the correct implementation of the `CalculateExpectedPhotonsScatteredSystem`
    #[test]
    fn test_calculate_expected_photons_scattered_system() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());
        //We assume 16 beams with equal `RateCoefficient`s for this test
        let mut rc = RateCoefficient::<Strontium88_461>::default();
        rc.rate = 1_000_000.0;
        let mut tps = TotalPhotonsScattered::<Strontium88_461>::default();
        tps.total = 8.0;

        let atom1 = app
            .world_mut()
            .spawn(tps)
            .insert(CoolingLaserSamplerMasks {
                contents: [CoolingLaserSamplerMask { filled: true }; LASER_COUNT],
            })
            .insert(RateCoefficients {
                contents: [rc; LASER_COUNT],
            })
            .insert(ExpectedPhotonsScatteredVector {
                contents: [ExpectedPhotonsScattered::<Strontium88_461>::default(); LASER_COUNT],
            })
            .id();

        app.add_systems(Update, calculate_expected_photons_scattered::<LASER_COUNT, Strontium88_461>);
        app.update();

        let scattered = 8.0 / LASER_COUNT as f64;

        assert_approx_eq!(
            app.world()
                .entity(atom1)
                .get::<ExpectedPhotonsScatteredVector<Strontium88_461, LASER_COUNT>>()
                .expect("entity not found")
                .contents[1] //all entries are equal
                .scattered,
            scattered,
            1e-5_f64
        );
    }
}
