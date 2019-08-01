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

// TODO: System which initialises array of LaserIntensitySamplers.