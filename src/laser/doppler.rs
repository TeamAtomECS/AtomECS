//! Calculations of the Doppler shift.
extern crate rayon;
extern crate specs;

use super::cooling::{CoolingLight, CoolingLightIndex};
use super::gaussian::GaussianBeam;
use crate::atom::Velocity;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

const LASER_CACHE_SIZE: usize = 16;

/// Represents the Dopplershift of the atom with respect to each beam due to the atom velocity
#[derive(Clone, Copy)]
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
pub struct CalculateDopplerShiftSystem;
impl<'a> System<'a> for CalculateDopplerShiftSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        WriteStorage<'a, DopplerShiftSamplers>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (cooling, indices, gaussian, mut samplers, velocities): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = (CoolingLight, CoolingLightIndex, GaussianBeam);
        let laser_cache: Vec<CachedLaser> = (&cooling, &indices, &gaussian)
            .join()
            .map(|(cooling, index, gaussian)| (cooling.clone(), index.clone(), gaussian.clone()))
            .collect();

        // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
        for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
            let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
            let slice = &laser_cache[base_index..max_index];
            let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
            laser_array[..max_index].copy_from_slice(slice);
            let number_in_iteration = slice.len();

            (&mut samplers, &velocities)
                .par_join()
                .for_each(|(sampler, vel)| {
                    for i in 0..number_in_iteration {
                        let (cooling, index, gaussian) = laser_array[i];
                        sampler.contents[index.index].doppler_shift = vel
                            .vel
                            .dot(&(gaussian.direction.normalize() * cooling.wavenumber()));
                    }
                })
        }
    }
}

/// Component that holds a list of `DopplerShiftSampler`s
///
/// Each list entry corresponds to the detuning with respect to a CoolingLight entity
/// and is indext via `CoolingLightIndex`
pub struct DopplerShiftSamplers {
    /// List of all `DopplerShiftSampler`s
    pub contents: [DopplerShiftSampler; crate::laser::COOLING_BEAM_LIMIT],
}
impl Component for DopplerShiftSamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all `DopplerShiftSamplers` to a NAN value.
///
/// It also ensures that the size of the `DopplerShiftSamplers` components match the number of CoolingLight entities in the world.
pub struct InitialiseDopplerShiftSamplersSystem;
impl<'a> System<'a> for InitialiseDopplerShiftSamplersSystem {
    type SystemData = (WriteStorage<'a, DopplerShiftSamplers>,);
    fn run(&mut self, (mut samplers,): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut samplers).par_join().for_each(|mut sampler| {
            sampler.contents = [DopplerShiftSampler::default(); crate::laser::COOLING_BEAM_LIMIT];
        });
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
    use crate::laser::gaussian;
    use nalgebra::Vector3;

    #[test]
    fn test_calculate_doppler_shift_system() {
        let mut test_world = World::new();
        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<Velocity>();
        test_world.register::<DopplerShiftSamplers>();

        let wavelength = 780e-9;
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
                rayleigh_range: gaussian::calculate_rayleigh_range(&wavelength, &2.0),
            })
            .build();

        let atom_velocity = 100.0;
        let sampler1 = test_world
            .create_entity()
            .with(Velocity {
                vel: Vector3::new(atom_velocity, 0.0, 0.0),
            })
            .with(DopplerShiftSamplers {
                contents: [DopplerShiftSampler::default(); crate::laser::COOLING_BEAM_LIMIT],
            })
            .build();

        let mut system = CalculateDopplerShiftSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<DopplerShiftSamplers>();

        assert_approx_eq!(
            sampler_storage
                .get(sampler1)
                .expect("entity not found")
                .contents[0]
                .doppler_shift,
            2.0 * PI / wavelength * atom_velocity,
            1e-5_f64
        );
    }
}
