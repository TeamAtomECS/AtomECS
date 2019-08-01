extern crate specs;
use specs::{
	DispatcherBuilder, World, Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage, ReadExpect, HashMapStorage, Entity
};

use crate::atom::{Position};
use crate::maths;
use rand::Rng;
use super::cooling::{CoolingLight};

/// A component representing a beam with a gaussian intensity profile.
pub struct GaussianBeam {
	
	/// A point that the laser beam intersects
	pub intersection: [f64;3],

	/// Direction the beam propagates with respect to cartesian `x,y,z` axes.
	pub direction: [f64;3],

	/// Radius of the beam at which the intensity is 1/e of the peak value, SI units of m. 
	pub e_radius: f64,

	/// Power of the laser in W
	pub power: f64,

}
impl Component for GaussianBeam {
	type Storage = HashMapStorage<Self>;
}

/// System that calculates the samples the intensity of `GaussianBeam` entities.
pub struct CalculateGaussianBeamIntensitySystem;
impl <'a> System<'a> for CalculateGaussianBeamIntensitySystem {
	type SystemData = (
        Entities<'a>,
        ReadStorage<'a,CoolingLight>,
        ReadStorage<'a,GaussianBeam>,
        WriteStorage<'a,LaserIntensitySamplers>,
        ReadStorage<'a,Position>
        );
	fn run (&mut self,(entities, cooling, gaussian, intensities):Self::SystemData){
		
        let mut iter=0;
        for (laser,cooling) in (&entities, &cooling).join() {
                
            // Perform only for Gaussian lasers
            let g = gaussian.get(laser);
            if (g.is_none) {
                continue;
            }

            for (pos, mut samplers) in (&pos, &samplers)
            {
                samplers.content[iter] = 
            }
        }
	}
}

}