extern crate rayon;
extern crate specs;

extern crate rand;
use rand::distributions::{Distribution, Poisson};

use crate::atom::AtomicTransition;
use crate::integrator::Timestep;
use crate::laser::cooling::CoolingLight;
use crate::laser::cooling::CoolingLightIndex;
use crate::laser::rate::RateCoefficients;
use crate::laser::twolevel::TwoLevelPopulation;
use specs::{Component, Join, ReadExpect, ReadStorage, System, VecStorage, WriteStorage};

use crate::constant::PI;

#[derive(Clone)]
pub struct TotalPhotonsScattered {
    pub total: f64,
}

impl Default for TotalPhotonsScattered {
    fn default() -> Self {
        TotalPhotonsScattered { total: f64::NAN }
    }
}

impl Component for TotalPhotonsScattered {
    type Storage = VecStorage<Self>;
}

/// Calcutates the total numer of Photons scattered in one iteration step
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
        for (atominfo, twolevel, total) in (
            &atomic_transition,
            &twolevel_population,
            &mut total_photons_scattered,
        )
            .join()
        {
            total.total = timestep.delta * (2. * PI * atominfo.linewidth) * twolevel.excited;
        }
    }
}

/// The number of photons scattered by the atom from a single, specific beam
#[derive(Clone)]
pub struct ExpectedPhotonsScattered {
    scattered: f64,
}

impl Default for ExpectedPhotonsScattered {
    fn default() -> Self {
        ExpectedPhotonsScattered {
            scattered: f64::NAN,
        }
    }
}

/// The List that holds a ExpectedPhotonsScattered for each laser
pub struct ExpectedPhotonsScatteredVector {
    pub contents: Vec<ExpectedPhotonsScattered>,
}

impl Component for ExpectedPhotonsScatteredVector {
    type Storage = VecStorage<Self>;
}

/// This system initialises all ExpectedPhotonsScatteredVector to a NAN value.
///
/// It also ensures that the size of the ExpectedPhotonsScatteredVector components match the number of CoolingLight entities in the world.
pub struct InitialiseExpectedPhotonsScatteredVectorSystem;
impl<'a> System<'a> for InitialiseExpectedPhotonsScatteredVectorSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, ExpectedPhotonsScatteredVector>,
    );
    fn run(&mut self, (cooling, cooling_index, mut expected_photons): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(ExpectedPhotonsScattered::default());
        }

        for mut expected in (&mut expected_photons).join() {
            expected.contents = content.clone();
        }
    }
}

/// Calcutates the expected mean number of Photons scattered by each laser in one iteration step
pub struct CalculateExpectedPhotonsScatteredSystem;
impl<'a> System<'a> for CalculateExpectedPhotonsScatteredSystem {
    type SystemData = (
        ReadStorage<'a, RateCoefficients>,
        ReadStorage<'a, TotalPhotonsScattered>,
        WriteStorage<'a, ExpectedPhotonsScatteredVector>,
    );

    fn run(
        &mut self,
        (rate_coefficients, total_photons_scattered, mut expected_photons_vector): Self::SystemData,
    ) {
        for (rates, total, expected) in (
            &rate_coefficients,
            &total_photons_scattered,
            &mut expected_photons_vector,
        )
            .join()
        {
            let mut sum_rates: f64 = 0.;

            for count in 0..rates.contents.len() {
                sum_rates = sum_rates + rates.contents[count].rate;
            }

            for index in 0..expected.contents.len() {
                expected.contents[index].scattered =
                    rates.contents[index].rate / sum_rates * total.total;
            }
        }
    }
}

/// The number of photons actually scattered by the atom from a single, specific beam
#[derive(Clone)]
pub struct ActualPhotonsScattered {
    scattered: u64,
}

impl Default for ActualPhotonsScattered {
    fn default() -> Self {
        ActualPhotonsScattered {
            scattered: u64::MIN,
        }
    }
}

/// The List that holds a ExpectedPhotonsScattered for each laser
pub struct ActualPhotonsScatteredVector {
    pub contents: Vec<ActualPhotonsScattered>,
}

impl Component for ActualPhotonsScatteredVector {
    type Storage = VecStorage<Self>;
}

/// This system initialises all ActualPhotonsScatteredVector to a NAN value.
///
/// It also ensures that the size of the ActualPhotonsScatteredVector components match the number of CoolingLight entities in the world.
pub struct InitialiseActualPhotonsScatteredVectorSystem;
impl<'a> System<'a> for InitialiseActualPhotonsScatteredVectorSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, ActualPhotonsScatteredVector>,
    );
    fn run(&mut self, (cooling, cooling_index, mut actual_photons): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(ActualPhotonsScattered::default());
        }

        for mut actual in (&mut actual_photons).join() {
            actual.contents = content.clone();
        }
    }
}

/// Calcutates the actual number of Photons scattered by each laser in one iteration step
/// by drawing from a Poisson Distribution
pub struct CalculateActualPhotonsScatteredSystem;
impl<'a> System<'a> for CalculateActualPhotonsScatteredSystem {
    type SystemData = (
        ReadStorage<'a, ExpectedPhotonsScatteredVector>,
        WriteStorage<'a, ActualPhotonsScatteredVector>,
    );

    fn run(&mut self, (expected_photons_vector, mut actual_photons_vector): Self::SystemData) {
        for (expected, actual) in (&expected_photons_vector, &mut actual_photons_vector).join() {
            for index in 0..expected.contents.len() {
                let poisson = Poisson::new(expected.contents[index].scattered);
                actual.contents[index].scattered = poisson.sample(&mut rand::thread_rng());
            }
        }
    }
}
