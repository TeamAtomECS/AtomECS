//! Calculation of RateCoefficients for the rate equation approach

extern crate rayon;
extern crate specs;

use super::cooling::{CoolingLight, CoolingLightIndex};
use crate::atom::AtomicTransition;
use crate::laser::gaussian::GaussianBeam;
use crate::laser::intensity::LaserIntensitySamplers;
use crate::laser::sampler::LaserDetuningSamplers;
use crate::magnetic::MagneticFieldSampler;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

use crate::constant::PI;

/// Represents the rate coefficient of the atom with respect to a specific CoolingLight entity
#[derive(Clone)]
pub struct RateCoefficient {
    /// rate coefficient in Hz
    pub rate: f64,
}

impl Default for RateCoefficient {
    fn default() -> Self {
        RateCoefficient {
            /// rate coefficient in Hz
            rate: f64::NAN,
        }
    }
}

/// Component that holds a Vector of `RateCoefficient`
pub struct RateCoefficients {
    /// Vector of `RateCoefficient` where each entry corresponds to a different CoolingLight entity
    pub contents: Vec<RateCoefficient>,
}
impl Component for RateCoefficients {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `RateCoefficient` to a NAN value.
///
/// It also ensures that the size of the `RateCoefficient` components match the number of CoolingLight entities in the world.
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

/// Calculates the TwoLevel approach rate coefficients for all atoms for all
/// CoolingLight entities
///
/// The Rate can be calculated by: Intensity * Absorption_Cross_Section / Photon_Energy
///
/// This is also the System that currently takes care of handling the polarizations correctly.
/// The polarization is projected onto the quantization axis given by the local magnetic
/// field vector. For fully polarized CoolingLight all projection pre-factors add up to 1.
pub struct CalculateRateCoefficientsSystem;

impl<'a> System<'a> for CalculateRateCoefficientsSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, LaserDetuningSamplers>,
        ReadStorage<'a, LaserIntensitySamplers>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, MagneticFieldSampler>,
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
            gaussian_beam,
            magnetic_field_sampler,
            mut rate_coefficients,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (cooling, index, gaussian) in (&cooling_light, &cooling_index, &gaussian_beam).join() {
            (
                &laser_detunings,
                &laser_intensities,
                &atomic_transition,
                &magnetic_field_sampler,
                &mut rate_coefficients,
            )
                .par_join()
                .for_each(|(detunings, intensities, atominfo, bfield, rates)| {
                    let beam_direction_vector = gaussian.direction.normalize();
                    let costheta = if &bfield.field.norm_squared() < &(10.0 * f64::EPSILON) {
                        0.0
                    } else {
                        beam_direction_vector
                            .normalize()
                            .dot(&bfield.field.normalize())
                    };

                    let prefactor =
                        atominfo.rate_prefactor * intensities.contents[index.index].intensity;

                    let scatter1 =
                        0.25 * (cooling.polarization as f64 * costheta + 1.).powf(2.) * prefactor
                            / (detunings.contents[index.index].detuning_sigma_plus.powf(2.)
                                + (PI * atominfo.linewidth).powf(2.));

                    let scatter2 =
                        0.25 * (cooling.polarization as f64 * costheta - 1.).powf(2.) * prefactor
                            / (detunings.contents[index.index]
                                .detuning_sigma_minus
                                .powf(2.)
                                + (PI * atominfo.linewidth).powf(2.));

                    let scatter3 = 0.5 * (1. - costheta.powf(2.)) * prefactor
                        / (detunings.contents[index.index].detuning_pi.powf(2.)
                            + (PI * atominfo.linewidth).powf(2.));
                    rates.contents[index.index].rate = scatter1 + scatter2 + scatter3;
                });
        }
    }
}
