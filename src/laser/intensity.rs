extern crate specs;
use specs::{Entities, ReadStorage, WriteStorage, Entity,VecStorage,Component,RunNow,System,Join};
use crate::laser::cooling::{CoolingLight,CoolingLightIndex};

/// Represents a sample of laser beam intensity
pub struct LaserIntensitySampler {
	/// Intensity of the laser beam, in SI units of Watts per metre
	pub intensity: f64
}
impl Clone for LaserIntensitySampler {
    fn clone(&self) -> Self { LaserIntensitySampler{ intensity: self.intensity }}
}

/// Component that holds a list of laser intensity samplers
pub struct LaserIntensitySamplers {
    /// List of laser intensity samplers
    pub contents: Vec<LaserIntensitySampler>
}
impl Component for LaserIntensitySamplers {
    type Storage = VecStorage<Self>;
}

/// This system initialises all LaserIntensitySamplers to a zero value.
/// 
/// It also ensures that the size of the LaserIntensitySamplers components match the number of CoolingLight entities in the world.
pub struct InitialiseLaserIntensitySamplersSystem;
impl <'a> System<'a> for InitialiseLaserIntensitySamplersSystem {
	type SystemData = (
        ReadStorage<'a,CoolingLight>,
        ReadStorage<'a,CoolingLightIndex>,
        WriteStorage<'a,LaserIntensitySamplers>,
        );
	fn run (&mut self,(cooling, cooling_index, mut intensity_samplers):Self::SystemData){
        let mut content = Vec::new();
        for (_,_) in (&cooling, &cooling_index).join() {
            content.push(
                LaserIntensitySampler { intensity: 0.0 }
            );
        }

        for mut intensity_sampler in (&mut intensity_samplers).join() {
            intensity_sampler.contents = content.clone();
        }
	}
}

//Pattern idea: detect when a new cooling laser is added, build a table of laser v index that can be iterated over. 
// First impl - create the laser list every frame. 
// Second impl - update the laser list only when new entity is added to storage.