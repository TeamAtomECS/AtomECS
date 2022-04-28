//! Calculation of [RateCoefficients] used in the rate equation formalism of laser cooling.

extern crate serde;

use std::marker::PhantomData;

use super::CoolingLight;
use super::transition::{TransitionComponent};
use crate::integrator::BatchSize;
use crate::laser::gaussian::GaussianBeam;
use crate::laser::index::LaserIndex;
use crate::laser::intensity::LaserIntensitySamplers;
use crate::laser_cooling::sampler::LaserDetuningSamplers;
use crate::magnetic::MagneticFieldSampler;
use bevy::tasks::ComputeTaskPool;
use serde::Serialize;
use bevy::prelude::*;

/// Represents the rate coefficient of the atom with respect to a specific [CoolingLight] entity, for the given transition.
#[derive(Clone, Copy, Serialize)]
pub struct RateCoefficient<T> where T : TransitionComponent {
    /// rate coefficient in Hz
    pub rate: f64,
    phantom: PhantomData<T>
}
impl<T> Default for RateCoefficient<T> where T : TransitionComponent {
    fn default() -> Self {
        RateCoefficient {
            /// rate coefficient in Hz
            rate: f64::NAN,
            phantom: PhantomData
        }
    }
}

/// Component that holds a Vector of `RateCoefficient`
#[derive(Clone, Copy, Serialize, Component)]
pub struct RateCoefficients<T, const N: usize> where T : TransitionComponent {
    /// Vector of `RateCoefficient` where each entry corresponds to a different CoolingLight entity
    #[serde(with = "serde_arrays")]
    pub contents: [RateCoefficient<T>; N],
}

/// Calculates the TwoLevel approach rate coefficients for all atoms for all
/// CoolingLight entities
///
/// The Rate can be calculated by: Intensity * Absorption_Cross_Section / Photon_Energy
///
/// This is also the System that currently takes care of handling the polarizations correctly.
/// The polarization is projected onto the quantization axis given by the local magnetic
/// field vector. For fully polarized CoolingLight all projection pre-factors add up to 1.
pub fn calculate_rate_coefficients<const N: usize, T>(
    laser_query: Query<(&CoolingLight, &LaserIndex, &GaussianBeam)>,
    mut atom_query: Query<(&LaserDetuningSamplers<T,N>, &LaserIntensitySamplers<N>, &MagneticFieldSampler, &mut RateCoefficients<T,N>), With<T>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>
) where T : TransitionComponent {

    // First set all rate coefficients to zero.
    atom_query.par_for_each_mut(&task_pool, batch_size.0,
        |(_, _, _, mut rates)| {
            rates.contents = [RateCoefficient::default(); N];
        }
    );

    // Then calculate for each laser.
    for (cooling, index, gaussian) in laser_query.iter() {
        atom_query.par_for_each_mut(
            &task_pool,
            batch_size.0,
            |(detunings, intensities, bfield, mut rates)| {
                let beam_direction_vector = gaussian.direction.normalize();
                let costheta = if bfield.field.norm_squared() < (10.0 * f64::EPSILON) {
                    0.0
                } else {
                    beam_direction_vector
                        .normalize()
                        .dot(&bfield.field.normalize())
                };

                let prefactor =
                    T::rate_prefactor() * intensities.contents[index.index].intensity;
                let gamma = T::gamma();

                let scatter1 =
                    0.25 * (cooling.polarization as f64 * costheta + 1.).powf(2.) * prefactor
                        / (detunings.contents[index.index].detuning_sigma_plus.powi(2)
                            + (gamma / 2.0).powi(2));

                let scatter2 =
                    0.25 * (cooling.polarization as f64 * costheta - 1.).powi(2) * prefactor
                        / (detunings.contents[index.index].detuning_sigma_minus.powi(2)
                            + (gamma / 2.0).powi(2));

                let scatter3 = 0.5 * (1. - costheta.powf(2.)) * prefactor
                    / (detunings.contents[index.index].detuning_pi.powi(2)
                        + (gamma / 2.0).powi(2));
                rates.contents[index.index].rate = scatter1 + scatter2 + scatter3;
            }
        );
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    use crate::laser::index::LaserIndex;
    use crate::laser::DEFAULT_BEAM_LIMIT;
    use crate::laser_cooling::CoolingLight;
    use crate::laser_cooling::transition::AtomicTransition;
    use crate::species::Strontium88_461;
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;
    use nalgebra::{Matrix3, Vector3};

    use crate::laser::intensity::{LaserIntensitySamplers, LaserIntensitySampler};
    use crate::laser_cooling::sampler::{LaserDetuningSamplers, LaserDetuningSampler};
    use crate::magnetic::MagneticFieldSampler;

    /// Tests the correct implementation of the `RateCoefficients`
    #[test]
    fn test_calculate_rate_coefficients_system() {
        let mut test_world = World::new();

        test_world.register::<LaserIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<LaserDetuningSamplers<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<LaserIntensitySamplers<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Strontium88_461>();
        test_world.register::<MagneticFieldSampler>();
        test_world.register::<RateCoefficients<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();

        let wavelength = 461e-9;
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength,
            })
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
                rayleigh_range: 1.0,
                ellipticity: 0.0,
            })
            .build();

        let detuning = -1.0e7;
        let field = Vector3::new(0.0, 0.0, 1.0);
        let intensity = 1.0;

        let mut lds = LaserDetuningSampler::<Strontium88_461>::default();
        lds.detuning_sigma_plus = detuning;
        lds.detuning_sigma_minus = detuning;
        lds.detuning_pi = detuning;

        let atom1 = test_world
            .create_entity()
            .with(LaserDetuningSamplers {
                contents: [lds; DEFAULT_BEAM_LIMIT],
            })
            .with(LaserIntensitySamplers {
                contents: [LaserIntensitySampler { intensity };
                    DEFAULT_BEAM_LIMIT],
            })
            .with(Strontium88_461)
            .with(MagneticFieldSampler {
                field,
                magnitude: 1.0,
                gradient: Vector3::new(0.0, 0.0, 0.0),
                jacobian: Matrix3::zeros(),
            })
            .with(RateCoefficients {
                contents: [RateCoefficient::<Strontium88_461>::default(); crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .build();

        let mut system = CalculateRateCoefficientsSystem::<Strontium88_461, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<RateCoefficients<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();

        let man_pref = Strontium88_461::rate_prefactor() * intensity;
        let scatter1 = 0.25 * man_pref
            / (detuning.powf(2.0) + (Strontium88_461::gamma() / 2.).powf(2.0));
        let scatter2 = 0.25 * man_pref
            / (detuning.powf(2.0) + (Strontium88_461::gamma() / 2.).powf(2.0));
        let scatter3 = 0.5 * man_pref
            / (detuning.powf(2.) + (Strontium88_461::gamma() / 2.).powf(2.));

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
