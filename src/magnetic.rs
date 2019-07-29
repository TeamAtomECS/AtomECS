extern crate specs;
use specs::{System,ReadStorage,WriteStorage,Join,Component,VecStorage,HashMapStorage};
use crate::atom::Position;
use crate::maths;

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
	/// Gradient of the quadrupole field, in units of Gauss/cm
	pub gradient:f64
}

impl Component for QuadrupoleField3D{
	type Storage = HashMapStorage<Self>;
}

/// A component representing a uniform bias field, of the form `B = [ B_x, B_y, B_z ]`
pub struct UniformMagneticField { 
	/// Vector field components with respect to the x,y,z cartesian axes, in units of Tesla.
	pub field:[f64;3],
}

impl Component for UniformMagneticField {
	type Storage = HashMapStorage<Self>;
}

impl UniformMagneticField {
	/// Create a UniformMagneticField with components specified in units of Gauss.
	pub fn gauss(components:[f64;3]) -> UniformMagneticField{ 
		UniformMagneticField{field: maths::array_multiply(&components,1e-4)}
		}

	/// Create a UniformMagneticField with components specified in units of Tesla.
	pub fn tesla(components:[f64;3]) -> UniformMagneticField{ 
		UniformMagneticField{field: components}
	}
}

/// Updates the values of magnetic field samplers to include quadrupole fields in the world.
pub struct Sample3DQuadrupoleFieldSystem;

impl Sample3DQuadrupoleFieldSystem {

	/// Calculates the quadrupole magnetic field.
	/// The field is defined with components `Bx = grad*x`, `By = grad*y`, `Bz = -2 * grad * z`.
	/// 
	/// # Arguments
	/// 
	/// `pos`: position of the sampler, m
	/// 
	/// `centre`: position of the quadrupole node, m
	/// 
	/// `gradient`: quadrupole gradient, in Tesla/m
	pub fn calculate_field(pos:&[f64;3], centre:&[f64;3], gradient:f64) -> [f64;3]{
		let rel_pos = maths::array_subtraction(&pos, &centre);
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
				let quad_field = Sample3DQuadrupoleFieldSystem::calculate_field(&pos.pos, &centre.pos, quadrupole.gradient);
				sampler.field = maths::array_addition(&quad_field, &sampler.field);
			}
		}
	}
}

/// Updates the values of magnetic field samplers to include uniform magnetic fields in the world.
pub struct UniformMagneticFieldSystem;

impl <'a> System<'a> for UniformMagneticFieldSystem{
		type SystemData = (WriteStorage<'a,MagneticFieldSampler>,
									ReadStorage<'a,UniformMagneticField>,
									);
	fn run(&mut self,(mut _sampler,fields):Self::SystemData){
		for field in (&fields).join() {
			for mut sampler in (&mut _sampler).join(){
				sampler.field = maths::array_addition(&sampler.field, &field.field);
			}
		}
	}
}

/// System that clears the magnetic field samplers each frame.
pub struct ClearMagneticFieldSamplerSystem;

impl <'a> System<'a> for ClearMagneticFieldSamplerSystem{
	type SystemData = (WriteStorage<'a,MagneticFieldSampler>);
	fn run (&mut self,mut _sampler:Self::SystemData){
		for sampler in (&mut _sampler).join(){
			sampler.magnitude = 0.;
			sampler.field = [0.,0.,0.]
		}
	}
}

/// System that calculates the magnitude of the magnetic field.
/// 
/// The magnetic field magnitude is frequently used, so it makes sense to calculate it once and cache the result.
/// This system runs after all other magnetic field systems.
pub struct CalculateMagneticFieldMagnitudeSystem;

impl <'a> System<'a> for CalculateMagneticFieldMagnitudeSystem {
	type SystemData = (WriteStorage<'a,MagneticFieldSampler>);
	fn run (&mut self,mut _sampler:Self::SystemData){
		for sampler in (&mut _sampler).join(){
			sampler.magnitude = maths::modulus(&sampler.field);
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