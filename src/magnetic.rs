extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,Component,VecStorage,HashMapStorage};
use crate::atom::Position;
use crate::maths::Maths;

/// A component that measures the magnetic field at a point in space.
pub struct MagneticFieldSampler{
	
	/// Vector representing the magnetic field components along x,y,z in units of Gauss.
	pub field:[f64;3],

	/// Magnitude of the magnetic field in units of Gauss
	pub magnitude: f64
}

impl Component for MagneticFieldSampler{
	type Storage = VecStorage<Self>;
}

/// A component representing a 3D quadrupole field.
pub struct QuadrupoleField3D{
	/// Gradient of the quadrupole field, in units of Tesla/m
	pub gradient:f64
}

impl Component for QuadrupoleField3D{
	type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include quadrupole fields in the world.
pub struct Sample3DQuadrupoleFieldSystem;

impl Sample3DQuadrupoleFieldSystem {

	/// Calculates the quadrupole magnetic field in units of Gauss.
	/// The field is defined with components `Bx = grad*x`, `By = grad*y`, `Bz = -2 * grad * z`.
	/// 
	/// # Arguments
	/// 
	/// `pos`: position of the sampler, m
	/// 
	/// `centre`: position of the quadrupole node, m
	/// 
	/// `gradient`: quadrupole gradient, in G/cm
	pub fn calculate_field(pos:&[f64;3], centre:&[f64;3], gradient:f64) -> [f64;3]{
		let rel_pos = Maths::array_subtraction(&pos, &centre);
		[rel_pos[0]*gradient, rel_pos[1]*gradient, rel_pos[2]*-2.*gradient]
	}
}

impl <'a> System<'a> for Sample3DQuadrupoleFieldSystem{
		type SystemData = (WriteStorage<'a,MagneticFieldSampler>,
									ReadStorage<'a,Position>,
									ReadStorage<'a,QuadrupoleField3D>,
									);
	fn run(&mut self,(mut _sampler,pos,_quadrupoles):Self::SystemData){
		for (centre, quadrupole) in (&pos, &_quadrupoles).join(){
			for (pos,mut sampler) in (&pos,&mut _sampler).join(){
				sampler.field = Sample3DQuadrupoleFieldSystem::calculate_field(&pos.pos, &centre.pos, quadrupole.gradient);
			}
		}
	}
}

pub struct ClearSampler;

impl <'a> System<'a> for ClearSampler{
	type SystemData = (WriteStorage<'a,MagneticFieldSampler>);
	fn run (&mut self,mut _sampler:Self::SystemData){
		for sampler in (&mut _sampler).join(){
			sampler.magnitude = 0.;
			sampler.field = [0.,0.,0.]
		}
	}
}

pub struct MagMagnitude;

impl <'a> System<'a> for MagMagnitude{
	type SystemData = (WriteStorage<'a,MagneticFieldSampler>);
	fn run (&mut self,mut _sampler:Self::SystemData){
		for sampler in (&mut _sampler).join(){
			sampler.magnitude = Maths::modulus(&sampler.field);
		}
	}
}

#[cfg(test)]
pub mod tests {

	use super::*;

	/// Tests the correct implementation of the quadrupole 3D field
	#[test]
	fn test_quadrupole3dfield() {
		let pos = [1.,1.,1.];
		let centre = [0.,1.,0.];
		let gradient = 1.;
		let field = Sample3DQuadrupoleFieldSystem::calculate_field(&pos, &centre, gradient);
		assert_eq!(field, [1.,0.,-2.]);
	}

}