//! Calculation of RateCoefficients for the rate equation approach

extern crate rayon;
extern crate specs;

use super::cooling::{CoolingLight, CoolingLightIndex};
use crate::atom::AtomicTransition;
use crate::constant::PI;
use crate::laser::gaussian::GaussianBeam;
use crate::laser::intensity::LaserIntensitySamplers;
use crate::laser::sampler::LaserDetuningSamplers;
use crate::laser::sampler::LaserSamplerMasks;
use crate::magnetic::MagneticFieldSampler;
use specs::Join;
use specs::{Component, ReadStorage, System, VecStorage, WriteStorage};

/// Represents the rate coefficient of the atom with respect to a specific CoolingLight entity
#[derive(Clone, Copy)]
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
    pub contents: [RateCoefficient; crate::laser::COOLING_BEAM_LIMIT],
}
impl Component for RateCoefficients {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `RateCoefficient` to a NAN value.
///
/// It also ensures that the size of the `RateCoefficient` components match the number of CoolingLight entities in the world.
pub struct InitialiseRateCoefficientsSystem;
impl<'a> System<'a> for InitialiseRateCoefficientsSystem {
    type SystemData = (WriteStorage<'a, RateCoefficients>,);
    fn run(&mut self, (mut rate_coefficients,): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut rate_coefficients)
            .par_join()
            .for_each(|mut rate_coefficient| {
                rate_coefficient.contents =
                    [RateCoefficient::default(); crate::laser::COOLING_BEAM_LIMIT];
            });
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
        ReadStorage<'a, LaserDetuningSamplers>,
        ReadStorage<'a, LaserIntensitySamplers>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, LaserSamplerMasks>,
        WriteStorage<'a, RateCoefficients>,
    );
    fn run(
        &mut self,
        (
            laser_detunings,
            laser_intensities,
            atomic_transition,
            magnetic_field_sampler,
            masks,
            mut rate_coefficients,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (
            &laser_detunings,
            &laser_intensities,
            &atomic_transition,
            &magnetic_field_sampler,
            &masks,
            &mut rate_coefficients,
        )
            .par_join()
            .for_each(|(detunings, intensities, atominfo, bfield, masks, rates)| {
                let costhetas = intensities.contents.iter().map(|intensity| {
                    if &bfield.field.norm_squared() < &(10.0 * f64::EPSILON) {
                        0.0
                    } else {
                        intensity.direction.dot(&bfield.field.normalize())
                    }
                });

                // LLVM should auto vectorize this but does not!
                let mut rates_f = [0.0_f64; crate::laser::COOLING_BEAM_LIMIT];
                for (rate, ((dsp, dsm, dpi, dpol), (intensity, (costheta, mask_filled)))) in
                    rates_f.iter_mut().zip(
                        detunings
                            .contents
                            .iter()
                            .map(|d| {
                                (
                                    d.detuning_sigma_plus,
                                    d.detuning_sigma_minus,
                                    d.detuning_pi,
                                    d.polarization,
                                )
                            })
                            .zip(
                                intensities
                                    .contents
                                    .iter()
                                    .map(|s| s.intensity)
                                    .zip(costhetas.zip(masks.contents.iter().map(|m| m.filled))),
                            ),
                    )
                {
                    if !mask_filled {
                        continue;
                    }

                    let prefactor = atominfo.rate_prefactor * intensity;

                    let scatter1 = 0.25 * (dpol * costheta + 1.).powf(2.) * prefactor
                        / (dsp.powf(2.) + (PI * atominfo.linewidth).powf(2.));

                    let scatter2 = 0.25 * (dpol * costheta - 1.).powf(2.) * prefactor
                        / (dsm.powf(2.) + (PI * atominfo.linewidth).powf(2.));

                    let scatter3 = 0.5 * (1. - costheta.powf(2.)) * prefactor
                        / (dpi.powf(2.) + (PI * atominfo.linewidth).powf(2.));
                    *rate = scatter1 + scatter2 + scatter3;
                }
                for (rate, rate_f) in rates.contents.iter_mut().zip(rates_f.iter()) {
                    rate.rate = *rate_f;
                }
            });
    }
}
