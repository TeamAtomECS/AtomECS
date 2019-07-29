extern crate specs;
extern crate rand;

use specs::{System,ReadStorage,WriteStorage,Join,ReadExpect,WriteExpect};

pub struct Step{
	pub n : u64,
}

pub struct Timestep{
	pub t:f64,
}

use crate::atom::*;
use crate::maths;
use crate::constant;

/// # Euler Integration
/// 
/// The EulerIntegrationSystem integrates the classical equations of motion for particles using the euler method:
/// ```  

/// ```
/// This integrator is simple to implement but prone to integration error.
/// 
/// The timestep duration is specified by the ```Timestep``` system resource.
pub struct EulerIntegrationSystem;

impl <'a> System<'a> for EulerIntegrationSystem{
	type SystemData = (	WriteStorage<'a,Position>,
									WriteStorage<'a,Velocity>,
									ReadExpect<'a,Timestep>,
									WriteExpect<'a,Step>,
									ReadStorage<'a,Force>,
									ReadStorage<'a,Mass>
									);
		
	fn run(&mut self,(mut pos,mut vel,t,mut step,force,mass):Self::SystemData){
		
		step.n = step.n +1;
		for (mut vel,mut pos,force,mass) in (&mut vel,&mut pos,&force,&mass).join(){
			EulerUpdating(&mut vel,&mut pos,&force,&mass,t.t);
		}
	}
}
fn EulerUpdating(vel:&mut Velocity,pos:&mut Position,force:&Force,mass:&Mass,time:f64){
		vel.vel = maths::array_addition(&vel.vel,&maths::array_multiply(&force.force,1./(constant::AMU*mass.value)*time));
		pos.pos = maths::array_addition(&pos.pos,&maths::array_multiply(&vel.vel,time));
}
pub mod tests {

	use super::*;

	#[test]
	fn test_euler() {
		let mut pos = Position{pos:[1.,1.,1.]};
		let mut vel = Velocity{vel:[0.,1.,0.]};
		let time = 1.;
		let mass = Mass{value:1./constant::AMU};
		let force = Force{force:[0.,0.,1.]};
		EulerUpdating(&mut vel,&mut pos,&force,&mass,time);
		assert_eq!(vel.vel, [0.,1.,1.]);
		assert_eq!(pos.pos,[1.,2.,2.]);
	}

}