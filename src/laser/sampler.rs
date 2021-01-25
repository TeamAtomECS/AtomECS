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
pub struct LightWavePropertiesSampler {

    /// wavevector of the laser beam on the atom, in units of inverse m.
    pub wavevector: Vector3<f64>,

    /// Polarization of the cooling laser. See [CoolingLight](crate::laser::cooling::CoolingLight) for more info.
    pub polarization: f64,
}
impl Default for LightWavePropertiesSampler {
    fn default() -> Self {
        LightWavePropertiesSampler {
            polarization: f64::NAN,
            wavevector: Vector3::new(0., 0., 0.),
        }
    }
}

/// Component that holds a list of laser samplers
pub struct LightWavePropertiesSamplers {
    /// List of laser samplers
    pub contents: Vec<LightWavePropertiesSampler>,
}
impl Component for LightWavePropertiesSamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all samplers to a zero value.
///
/// It also ensures that the size of the LaserIntensitySamplers components match the number of CoolingLight entities in the world.
pub struct InitialiseLightWavePropertiesSamplersSystem;
impl<'a> System<'a> for InitialiseLightWavePropertiesSamplersSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LightWavePropertiesSamplers>,
    );
    fn run(&mut self, (cooling, cooling_index, mut light_samplers): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LightWavePropertiesSampler::default());
        }

        for mut light_sampler in (&mut light_samplers).join() {
            light_sampler.contents = content.clone();
        }
    }
}

/// Represents total detuning of the atom with respect to each beam
#[derive(Clone)]
pub struct LaserDetuningSampler {
    pub detuning: f64,
}

impl Default for LaserDetuningSampler {
    fn default() -> Self {
        LaserDetuningSampler {
            /// Laser detuning of all transitions with respect to laser beam, in SI units of Hz.
            detuning: f64::NAN,
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
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, LightWavePropertiesSamplers>,
        ReadStorage<'a, DopplerShiftSamplers>,
        ReadStorage<'a, ZeemanShiftSampler>,
        WriteStorage<'a, LaserDetuningSamplers>,
    );

    fn run(
        &mut self,
        (
            atom_info,
            indices,
            cooling_light,
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
        type CachedLaser = (CoolingLightIndex, CoolingLight);
        let laser_cache: Vec<CachedLaser> = (&indices, &cooling_light).join().map(|(index, cooling)| (index.clone(), cooling.clone())).collect();

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
                            let (index, cooling) = laser_array[i];
                            let without_zeeman =
                                (laser_samplers.contents[index.index].wavevector.norm()
                                    * constant::C
                                    / 2.
                                    / constant::PI
                                    - atom_info.frequency)
                                    * 2.0
                                    * constant::PI
                                    - doppler_samplers.contents[index.index].doppler_shift;

                            
                            detuning_sampler.contents[index.index].detuning = without_zeeman.clone() - match cooling.polarization {
                                1 => zeeman_sampler.sigma_plus,
                                -1 => zeeman_sampler.sigma_minus,
                                0 => zeeman_sampler.sigma_pi,
                                _ => panic!("The polarization provided did not match any of the accepted cases (CalculateLaserDetuningSystem)"),
                            };
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
        test_world.register::<LightWavePropertiesSamplers>();

        test_world
            .create_entity()
            .with(CoolingLightIndex::default())
            .with(CoolingLight {
                polarization: 1,
                wavelength: 780e-9,
            })
            .build();
        test_world
            .create_entity()
            .with(CoolingLightIndex::default())
            .with(CoolingLight {
                polarization: 1,
                wavelength: 780e-9,
            })
            .build();

        let test_sampler = test_world
            .create_entity()
            .with(LightWavePropertiesSamplers {
                contents: Vec::new(),
            })
            .build();

        let mut system = InitialiseLightWavePropertiesSamplersSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LightWavePropertiesSamplers>();
        let samplers = sampler_storage.get(test_sampler).expect("entity not found");
        assert_eq!(samplers.contents.len(), 2);
    }
}
