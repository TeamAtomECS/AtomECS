//! Calculation of [RateCoefficients] used in the rate equation formalism of laser cooling.

extern crate serde;

use std::marker::PhantomData;

use super::transition::TransitionComponent;
use super::CoolingLight;
use crate::integrator::AtomECSBatchStrategy;
use crate::laser::gaussian::GaussianBeam;
use crate::laser::index::LaserIndex;
use crate::laser::intensity::LaserIntensitySamplers;
use crate::laser_cooling::sampler::LaserDetuningSamplers;
use crate::magnetic::MagneticFieldSampler;
use bevy::prelude::*;
use serde::Serialize;

/// Represents the rate coefficient of the atom with respect to a specific [CoolingLight] entity, for the given transition.
#[derive(Clone, Copy, Serialize)]
pub struct RateCoefficient<T>
where
    T: TransitionComponent,
{
    // rate coefficient in Hz
    pub rate: f64,
    phantom: PhantomData<T>,
}
impl<T> Default for RateCoefficient<T>
where
    T: TransitionComponent,
{
    fn default() -> Self {
        RateCoefficient {
            // rate coefficient in Hz
            rate: f64::NAN,
            phantom: PhantomData,
        }
    }
}

/// Component that holds a Vector of `RateCoefficient`
#[derive(Clone, Copy, Serialize, Component)]
pub struct RateCoefficients<T, const N: usize>
where
    T: TransitionComponent,
{
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
    mut atom_query: Query<
        (
            &LaserDetuningSamplers<T, N>,
            &LaserIntensitySamplers<N>,
            &MagneticFieldSampler,
            &mut RateCoefficients<T, N>,
        ),
        With<T>,
    >,
    batch_strategy: Res<AtomECSBatchStrategy>,
) where
    T: TransitionComponent,
{
    // First set all rate coefficients to zero.
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(_, _, _, mut rates)| {
            rates.contents = [RateCoefficient::default(); N];
        });

    // Then calculate for each laser.
    for (cooling, index, gaussian) in laser_query.iter() {
        atom_query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(detunings, intensities, bfield, mut rates)| {
                let beam_direction_vector = gaussian.direction.normalize();
                let costheta = if bfield.field.norm_squared() < (10.0 * f64::EPSILON) {
                    0.0
                } else {
                    beam_direction_vector
                        .normalize()
                        .dot(&bfield.field.normalize())
                };

                let prefactor = T::rate_prefactor() * intensities.contents[index.index].intensity;
                let gamma = T::gamma();

                let scatter1 =
                    0.25 * (cooling.polarization as f64 * costheta + 1.).powi(2) * prefactor
                        / (detunings.contents[index.index].detuning_sigma_plus.powi(2)
                            + (gamma / 2.0).powi(2));

                let scatter2 =
                    0.25 * (cooling.polarization as f64 * costheta - 1.).powi(2) * prefactor
                        / (detunings.contents[index.index].detuning_sigma_minus.powi(2)
                            + (gamma / 2.0).powi(2));

                let scatter3 = 0.5 * (1. - costheta.powi(2)) * prefactor
                    / (detunings.contents[index.index].detuning_pi.powi(2) + (gamma / 2.0).powi(2));
                rates.contents[index.index].rate = scatter1 + scatter2 + scatter3;
            });
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    use crate::laser::index::LaserIndex;
    use crate::laser_cooling::transition::AtomicTransition;
    use crate::laser_cooling::CoolingLight;
    use crate::species::Strontium88_461;
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::{Matrix3, Vector3};

    use crate::laser::intensity::{LaserIntensitySampler, LaserIntensitySamplers};
    use crate::laser_cooling::sampler::{LaserDetuningSampler, LaserDetuningSamplers};
    use crate::magnetic::MagneticFieldSampler;

    const LASER_COUNT: usize = 4;

    /// Tests the correct implementation of the `RateCoefficients`
    #[test]
    fn test_calculate_rate_coefficients_system() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());
        let wavelength = 461e-9;
        app.world_mut()
            .spawn(CoolingLight {
                polarization: 1,
                wavelength,
            })
            .insert(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
                rayleigh_range: 1.0,
                ellipticity: 0.0,
            });

        let detuning = -1.0e7;
        let field = Vector3::new(0.0, 0.0, 1.0);
        let intensity = 1.0;

        let mut lds = LaserDetuningSampler::<Strontium88_461>::default();
        lds.detuning_sigma_plus = detuning;
        lds.detuning_sigma_minus = detuning;
        lds.detuning_pi = detuning;

        let atom1 = app
            .world_mut()
            .spawn(LaserDetuningSamplers {
                contents: [lds; LASER_COUNT],
            })
            .insert(LaserIntensitySamplers {
                contents: [LaserIntensitySampler { intensity }; LASER_COUNT],
            })
            .insert(Strontium88_461)
            .insert(MagneticFieldSampler {
                field,
                magnitude: 1.0,
                gradient: Vector3::new(0.0, 0.0, 0.0),
                jacobian: Matrix3::zeros(),
            })
            .insert(RateCoefficients {
                contents: [RateCoefficient::<Strontium88_461>::default(); LASER_COUNT],
            })
            .id();

        app.add_systems(Update, calculate_rate_coefficients::<LASER_COUNT, Strontium88_461>);
        app.update();

        let man_pref = Strontium88_461::rate_prefactor() * intensity;
        let scatter1 =
            0.25 * man_pref / (detuning.powf(2.0) + (Strontium88_461::gamma() / 2.).powf(2.0));
        let scatter2 =
            0.25 * man_pref / (detuning.powf(2.0) + (Strontium88_461::gamma() / 2.).powf(2.0));
        let scatter3 =
            0.5 * man_pref / (detuning.powf(2.) + (Strontium88_461::gamma() / 2.).powf(2.));

        assert_approx_eq!(
            app.world()
                .entity(atom1)
                .get::<RateCoefficients<Strontium88_461, LASER_COUNT>>()
                .expect("entity not found")
                .contents[0]
                .rate,
            scatter1 + scatter2 + scatter3,
            1e-5_f64
        );
    }
}
