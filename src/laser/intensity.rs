extern crate specs;
use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

/// Represents a sample of laser beam intensity
pub struct LaserIntensitySampler {
    /// Intensity of the laser beam, in SI units of Watts per metre
    pub intensity: f64,
}
impl Clone for LaserIntensitySampler {
    fn clone(&self) -> Self {
        LaserIntensitySampler {
            intensity: self.intensity,
        }
    }
}

/// Component that holds a list of laser intensity samplers
pub struct LaserIntensitySamplers {
    /// List of laser intensity samplers
    pub contents: Vec<LaserIntensitySampler>,
}
impl Component for LaserIntensitySamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all LaserIntensitySamplers to a zero value.
///
/// It also ensures that the size of the LaserIntensitySamplers components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserIntensitySamplersSystem;
impl<'a> System<'a> for InitialiseLaserIntensitySamplersSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        WriteStorage<'a, LaserIntensitySamplers>,
    );
    fn run(&mut self, (cooling, cooling_index, mut intensity_samplers): Self::SystemData) {
        let mut content = Vec::new();
        for (_, _) in (&cooling, &cooling_index).join() {
            content.push(LaserIntensitySampler { intensity: 0.0 });
        }

        for mut intensity_sampler in (&mut intensity_samplers).join() {
            intensity_sampler.contents = content.clone();
        }
    }
}

//Pattern idea: detect when a new cooling laser is added, build a table of laser v index that can be iterated over.
// First impl - create the laser list every frame.
// Second impl - update the laser list only when new entity is added to storage.

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
    use specs::{Builder, RunNow, World};

    #[test]
    fn test_initialise_laser_intensity_sampler_system() {
        let mut test_world = World::new();
        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<LaserIntensitySamplers>();

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
            .with(LaserIntensitySamplers {
                contents: Vec::new(),
            })
            .build();

        let mut system = InitialiseLaserIntensitySamplersSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserIntensitySamplers>();
        let samplers = sampler_storage.get(test_sampler).expect("entity not found");
        assert_eq!(samplers.contents.len(), 2);
        assert_eq!(samplers.contents[0].intensity, 0.0);
        assert_eq!(samplers.contents[1].intensity, 0.0);
    }
}
