extern crate specs;
use crate::atom::AtomicTransition;
use crate::constant;
use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
use crate::laser::doppler::DopplerShiftSamplers;
use crate::magnetic::zeeman::ZeemanShiftSampler;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};
use std::f64;
extern crate nalgebra;
use nalgebra::Vector3;

const LASER_CACHE_SIZE: usize = 16;

/// Represents a sample of a laser beam
#[derive(Clone)]
pub struct LaserSampler {
    /// Calculated force exerted by this laser sampler on the atom. Units of N.
    pub force: Vector3<f64>,

    /// Scattering rate of this laser sampler. Units of Hz, as in 'photons scattered per second'.
    pub scattering_rate: f64,

    /// Intensity of the laser beam, in SI units of Watts per metre
    pub intensity: f64,

    /// wavevector of the laser beam on the atom, in units of inverse m.
    pub wavevector: Vector3<f64>,

    /// Polarization of the cooling laser. See [CoolingLight](crate::laser::cooling::CoolingLight) for more info.
    pub polarization: f64,
}
impl Default for LaserSampler {
    fn default() -> Self {
        LaserSampler {
            force: Vector3::new(0., 0., 0.),
            polarization: f64::NAN,
            wavevector: Vector3::new(0., 0., 0.),
            intensity: f64::NAN,
            scattering_rate: f64::NAN,
        }
    }
}

/// Component that holds a list of laser samplers
pub struct LaserSamplers {
    /// List of laser samplers
    pub contents: Vec<LaserSampler>,
}
impl Component for LaserSamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all samplers to a zero value.
///
/// It also ensures that the size of the LaserIntensitySamplers components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserSamplersSystem;
impl<'a> System<'a> for InitialiseLaserSamplersSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LaserSamplers>,
    );
    fn run(&mut self, (cooling, cooling_index, mut intensity_samplers): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LaserSampler::default());
        }

        for mut intensity_sampler in (&mut intensity_samplers).join() {
            intensity_sampler.contents = content.clone();
        }
    }
}

/// Represents total detuning of the atom with respect to each beam
#[derive(Clone)]
pub struct LaserDetuningSampler {
    pub detuning_sigma_plus: f64,
    pub detuning_sigma_minus: f64,
    pub detuning_sigma_pi: f64,
}

impl Default for LaserDetuningSampler {
    fn default() -> Self {
        LaserDetuningSampler {
            /// Laser detuning of all transitions with respect to laser beam, in SI units of Hz.
            detuning_sigma_plus: f64::NAN,
            detuning_sigma_minus: f64::NAN,
            detuning_sigma_pi: f64::NAN,
        }
    }
}

/// Component that holds a list of laser detuning samplers
pub struct LaserDetuningSamplers {
    /// List of laser samplers
    pub contents: Vec<LaserDetuningSampler>,
}
impl Component for LaserDetuningSamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all LaserDetuningSamplers to a NAN value.
///
/// It also ensures that the size of the LaserDetuningSamplers components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserDetuningSamplersSystem;
impl<'a> System<'a> for InitialiseLaserDetuningSamplersSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LaserDetuningSamplers>,
    );
    fn run(&mut self, (cooling, cooling_index, mut samplers): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LaserDetuningSampler::default());
        }

        for mut sampler in (&mut samplers).join() {
            sampler.contents = content.clone();
        }
    }
}

/// This system calculates the total Laser Detuning for each atom in each cooling beam.
pub struct CalculateLaserDetuningSystem;
impl<'a> System<'a> for CalculateLaserDetuningSystem {
    type SystemData = (
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, DopplerShiftSamplers>,
        ReadStorage<'a, ZeemanShiftSampler>,
        WriteStorage<'a, LaserDetuningSamplers>,
    );

    fn run(
        &mut self,
        (
            atom_info,
            indices,
            laser_samplers,
            doppler_samplers,
            zeeman_sampler,
            mut detuning_samplers,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = CoolingLightIndex;
        let laser_cache: Vec<CachedLaser> = indices.join().map(|index| index.clone()).collect();

        // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
        for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
            let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
            let slice = &laser_cache[base_index..max_index];
            let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
            laser_array[..max_index].copy_from_slice(slice);
            let number_in_iteration = slice.len();

            (
                &mut detuning_samplers,
                &doppler_samplers,
                &zeeman_sampler,
                &atom_info,
                &laser_samplers,
            )
                .par_join()
                .for_each(
                    |(
                        detuning_sampler,
                        doppler_samplers,
                        zeeman_sampler,
                        atom_info,
                        laser_samplers,
                    )| {
                        for i in 0..number_in_iteration {
                            let index = laser_array[i];
                            let without_zeeman =
                                (laser_samplers.contents[index.index].wavevector.norm()
                                    * constant::C
                                    / 2.
                                    / constant::PI
                                    - atom_info.frequency)
                                    * 2.0
                                    * constant::PI
                                    - doppler_samplers.contents[index.index].doppler_shift;

                            detuning_sampler.contents[index.index].detuning_sigma_plus =
                                without_zeeman.clone() - zeeman_sampler.sigma_plus;
                            detuning_sampler.contents[index.index].detuning_sigma_minus =
                                without_zeeman.clone() - zeeman_sampler.sigma_minus;
                            detuning_sampler.contents[index.index].detuning_sigma_pi =
                                without_zeeman.clone() - zeeman_sampler.sigma_pi;
                        }
                    },
                )
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
    use specs::{Builder, RunNow, World};

    #[test]
    fn test_initialise_laser_sampler_system() {
        let mut test_world = World::new();
        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<LaserSamplers>();

        test_world
            .create_entity()
            .with(CoolingLightIndex::default())
            .with(CoolingLight {
                polarization: 1.0,
                wavelength: 780e-9,
            })
            .build();
        test_world
            .create_entity()
            .with(CoolingLightIndex::default())
            .with(CoolingLight {
                polarization: 1.0,
                wavelength: 780e-9,
            })
            .build();

        let test_sampler = test_world
            .create_entity()
            .with(LaserSamplers {
                contents: Vec::new(),
            })
            .build();

        let mut system = InitialiseLaserSamplersSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserSamplers>();
        let samplers = sampler_storage.get(test_sampler).expect("entity not found");
        assert_eq!(samplers.contents.len(), 2);
        assert_eq!(samplers.contents[0].intensity.is_nan(), true);
        assert_eq!(samplers.contents[1].intensity.is_nan(), true);
    }
}
