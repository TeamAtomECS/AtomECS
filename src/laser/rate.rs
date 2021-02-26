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

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::constant::PI;
    use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    use crate::laser::intensity::LaserIntensitySamplers;
    use crate::laser::sampler::LaserDetuningSamplers;
    use crate::magnetic::MagneticFieldSampler;

    /// Tests the correct implementation of the `RateCoefficients`
    #[test]
    fn test_calculate_rate_coefficients_system() {
        let mut test_world = World::new();

        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<LaserDetuningSamplers>();
        test_world.register::<LaserIntensitySamplers>();
        test_world.register::<AtomicTransition>();
        test_world.register::<MagneticFieldSampler>();
        test_world.register::<RateCoefficients>();

        let wavelength = 461e-9;
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength: wavelength,
            })
            .with(CoolingLightIndex {
                index: 0,
                initiated: true,
            })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
            })
            .build();

        let detuning = -1.0e7;
        let field = Vector3::new(0.0, 0.0, 1.0);
        let intensity = 1.0;

        let atom1 = test_world
            .create_entity()
            .with(LaserDetuningSamplers {
                contents: [crate::laser::sampler::LaserDetuningSampler {
                    detuning_sigma_plus: detuning,
                    detuning_sigma_minus: detuning,
                    detuning_pi: detuning,
                }; crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(LaserIntensitySamplers {
                contents: [crate::laser::intensity::LaserIntensitySampler {
                    intensity: intensity,
                }; crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(AtomicTransition::strontium())
            .with(MagneticFieldSampler {
                field: field,
                magnitude: 1.0,
            })
            .with(RateCoefficients {
                contents: [RateCoefficient::default(); crate::laser::COOLING_BEAM_LIMIT],
            })
            .build();

        let mut system = CalculateRateCoefficientsSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<RateCoefficients>();

        let man_pref = AtomicTransition::strontium().rate_prefactor * intensity;
        let scatter1 = 0.25 * man_pref
            / (detuning.powf(2.0) + (PI * AtomicTransition::strontium().linewidth).powf(2.0));
        let scatter2 = 0.25 * man_pref
            / (detuning.powf(2.0) + (PI * AtomicTransition::strontium().linewidth).powf(2.0));
        let scatter3 = 0.5 * man_pref
            / (detuning.powf(2.) + (PI * AtomicTransition::strontium().linewidth).powf(2.));

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .contents[0]
                .rate,
            scatter1 + scatter2 + scatter3,
            1e-5_f64
        );
    }
}
