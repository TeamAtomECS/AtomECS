extern crate rayon;
extern crate specs;

use crate::atom::AtomicTransition;
use crate::integrator::Timestep;
use crate::laser::twolevel::TwoLevelPopulation;
use specs::{Component, Join, ReadExpect, ReadStorage, System, VecStorage, WriteStorage};

use crate::constant::PI;

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
