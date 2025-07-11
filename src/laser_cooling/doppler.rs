//! Calculations of the Doppler shift.
use bevy::prelude::*;

use super::CoolingLight;
use crate::atom::Velocity;
use crate::integrator::AtomECSBatchStrategy;
use crate::laser::gaussian::GaussianBeam;
use crate::laser::index::LaserIndex;
use serde::Serialize;

const LASER_CACHE_SIZE: usize = 16;

/// Represents the Dopplershift of the atom with respect to each beam due to the atom velocity
#[derive(Clone, Copy, Serialize)]
pub struct DopplerShiftSampler {
    /// detuning value in rad/s
    pub doppler_shift: f64,
}
impl Default for DopplerShiftSampler {
    fn default() -> Self {
        DopplerShiftSampler {
            /// Doppler shift with respect to laser beam, in SI units of rad/s.
            doppler_shift: f64::NAN,
        }
    }
}

/// This system calculates the Doppler shift for each atom in each cooling beam.
///
/// The result is stored in `DopplerShiftSamplers`
pub fn calculate_doppler_shift<const N: usize>(
    laser_query: Query<(&CoolingLight, &LaserIndex, &GaussianBeam)>,
    mut atom_query: Query<(&mut DopplerShiftSamplers<N>, &Velocity)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    // Set samplers to default values first.
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut samplers, _vel)| {
            samplers.contents = [DopplerShiftSampler::default(); N];
        });

    // Enumerate through lasers and calculate for each.
    //
    // There are typically only a small number of lasers in a simulation.
    // For a speedup, cache the required components into thread memory,
    // so they can be distributed to parallel workers during the atom loop.
    type CachedLaser = (CoolingLight, LaserIndex, GaussianBeam);
    let mut laser_cache: Vec<CachedLaser> = Vec::new();
    for (&cooling, &index, &gaussian) in laser_query.iter() {
        laser_cache.push((cooling, index, gaussian));
    }

    // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
    for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
        let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
        let slice = &laser_cache[base_index..max_index];
        let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
        laser_array[..max_index].copy_from_slice(slice);
        let number_in_iteration = slice.len();

        atom_query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(mut sampler, vel)| {
                for (cooling, index, gaussian) in laser_array.iter().take(number_in_iteration) {
                    sampler.contents[index.index].doppler_shift = vel
                        .vel
                        .dot(&(gaussian.direction.normalize() * cooling.wavenumber()));
                }
            });
    }
}

/// Component that holds a list of [DopplerShiftSampler]s
///
/// Each list entry corresponds to the detuning with respect to a [CoolingLight] entity
/// and indexed via their [LaserIndex].
#[derive(Clone, Copy, Serialize, Component)]
pub struct DopplerShiftSamplers<const N: usize> {
    /// List of all `DopplerShiftSampler`s
    #[serde(with = "serde_arrays")]
    pub contents: [DopplerShiftSampler; N],
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::constant::PI;
    use crate::laser::gaussian;
    use crate::laser_cooling::CoolingLight;
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;

    #[test]
    fn test_calculate_doppler_shift_system() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());

        let wavelength = 780e-9;
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
                rayleigh_range: gaussian::calculate_rayleigh_range(&wavelength, &2.0),
                ellipticity: 0.0,
            });

        let atom_velocity = 100.0;
        let sampler1 = app
            .world_mut()
            .spawn(Velocity {
                vel: Vector3::new(atom_velocity, 0.0, 0.0),
            })
            .insert(DopplerShiftSamplers {
                contents: [DopplerShiftSampler::default(); 1],
            })
            .id();

        app.add_systems(Update, calculate_doppler_shift::<1>);
        app.update();

        assert_approx_eq!(
            app.world()
                .entity(sampler1)
                .get::<DopplerShiftSamplers<1>>()
                .expect("entity not found")
                .contents[0]
                .doppler_shift,
            2.0 * PI / wavelength * atom_velocity,
            1e-5_f64
        );
    }
}
