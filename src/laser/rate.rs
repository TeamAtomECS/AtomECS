extern crate rayon;
extern crate specs;

use super::cooling::{CoolingLight, CoolingLightIndex};
use crate::atom::AtomicTransition;
use crate::laser::intensity::LaserIntensitySamplers;
use crate::laser::sampler::LaserDetuningSamplers;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

use crate::constant::{C, HBAR, PI};

/// Represents the Rate Coefficient of the atom with respect to a certain laser beam
#[derive(Clone)]
pub struct RateCoefficient {
    pub rate: f64, // in Hz
}

impl Default for RateCoefficient {
    fn default() -> Self {
        RateCoefficient {
            rate: f64::NAN, // in Hz
        }
    }
}

/// Component that holds a list of laser intensity samplers
pub struct RateCoefficients {
    /// List of laser samplers
    pub contents: Vec<RateCoefficient>,
}
impl Component for RateCoefficients {
    type Storage = VecStorage<Self>;
}

/// This system initialises all RateCoefficient to a NAN value.
///
/// It also ensures that the size of the RateCoefficient components match the number of CoolingLight entities in the world.
pub struct InitialiseRateCoefficientsSystem;
impl<'a> System<'a> for InitialiseRateCoefficientsSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, RateCoefficients>,
    );
    fn run(&mut self, (cooling, cooling_index, mut rate_coefficients): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(RateCoefficient::default());
        }

        for mut rate_coefficient in (&mut rate_coefficients).join() {
            rate_coefficient.contents = content.clone();
        }
    }
}

pub struct CalculateRateCoefficientsSystem;

impl<'a> System<'a> for CalculateRateCoefficientsSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, LaserDetuningSamplers>,
        ReadStorage<'a, LaserIntensitySamplers>,
        ReadStorage<'a, AtomicTransition>,
        WriteStorage<'a, RateCoefficients>,
    );
    fn run(
        &mut self,
        (
            cooling_light,
            cooling_index,
            laser_detunings,
            laser_intensities,
            atomic_transition,
            mut rate_coefficients,
        ): Self::SystemData,
    ) {
        for (_cooling, index) in (&cooling_light, &cooling_index).join() {
            for (detunings, intensities, atominfo, rates) in (
                &laser_detunings,
                &laser_intensities,
                &atomic_transition,
                &mut rate_coefficients,
            )
                .join()
            {
                let prefactor = 3. / 4. * C.powf(2.) / HBAR
                    * (atominfo.frequency).powf(-3.)
                    * atominfo.linewidth;

                rates.contents[index.index].rate =
                    prefactor * intensities.contents[index.index].intensity * atominfo.linewidth
                        / (detunings.contents[index.index].detuning.powf(2.)
                            + (PI * atominfo.linewidth).powf(2.));
            }
        }
    }
}
