extern crate specs;
use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};
use std::f64;
extern crate nalgebra;
use nalgebra::Vector3;

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
    fn run(
        &mut self,
        (cooling, cooling_index, mut intensity_samplers): Self::SystemData,
    ) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LaserSampler::default());
        }

        for mut intensity_sampler in (&mut intensity_samplers).join() {
            intensity_sampler.contents = content.clone();
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
