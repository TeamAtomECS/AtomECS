extern crate rayon;
extern crate specs;

use crate::atom::AtomicTransition;
use crate::laser::rate::RateCoefficients;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

use crate::constant::PI;

/// Represents the steady-state population density of the excited state and ground state
pub struct TwoLevelPopulation {
    pub ground: f64,
    pub excited: f64,
}

impl Default for TwoLevelPopulation {
    fn default() -> Self {
        TwoLevelPopulation {
            ground: f64::NAN,
            excited: f64::NAN,
        }
    }
}

impl TwoLevelPopulation {
    /// Calculate the ground state population from excited state population
    pub fn calculate_ground_state(&self) {
        self.ground = 1. - self.excited;
    }
    /// Calculate the excited state population from ground state population
    pub fn calculate_excited_state(&self) {
        self.excited = 1. - self.ground;
    }
}

impl Component for TwoLevelPopulation {
    type Storage = VecStorage<Self>;
}

/// Calculates the TwoLevelPopulation from the natural linewidth and the rate coefficients
pub struct CalculateTwoLevelPopulation;
impl<'a> System<'a> for CalculateTwoLevelPopulation {
    type SystemData = (
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, RateCoefficients>,
        WriteStorage<'a, TwoLevelPopulation>,
    );

    fn run(
        &mut self,
        (atomic_transition, rate_coefficients, twolevel_population): Self::SystemData,
    ) {
        for (atominfo, rates, twolevel) in (
            &atomic_transition,
            &rate_coefficients,
            &mut twolevel_population,
        )
            .join()
        {
            let mut sum_rates: f64 = 0.;

            for count in 0..rates.contents.len() {
                sum_rates = sum_rates + rates.contents[count].rate;
            }
            // corresponds to equation 4 of paper draft
            twolevel.excited = sum_rates / (atominfo.linewidth * 2. * PI + 2. * sum_rates);

            // not currently used
            twolevel.calculate_ground_state();
        }
    }
}
