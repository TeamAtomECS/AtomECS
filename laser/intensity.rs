extern crate specs;
use specs::{Entity,VecStorage,Component};

use crate::maths;
use rand::Rng;

/// Represents a sample of laser beam intensity
struct LaserIntensitySampler {
	/// Laser associated with this sample
	pub laser: Entity,
	
	/// Intensity of the laser beam, in SI units of Watts per metre
	pub intensity: f64
}

/// Component that holds a list of laser intensity samplers
struct LaserIntensitySamplers {
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
        Entities<'a>,
        ReadStorage<'a,CoolingLight>,
        WriteStorage<'a,LaserIntensitySamplers>,
        );
	fn run (&mut self,(entities, cooling, mut intensity_samplers):Self::SystemData){
        let mut content = Vec::new();
        for (laser,cooling) in (&entities, &cooling).join() {
            content.push(
                LaserIntensitySampler { laser: laser, intensity: 0 }
            );
        }

        for mut intensity_sampler in (&mut intensity_samplers).join() {
            intensity_sampler.contents = content;
        }
	}
}