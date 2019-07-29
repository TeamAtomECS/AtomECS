extern crate specs;
extern crate rand;

use specs::{System,ReadStorage,WriteStorage,Join,ReadExpect,WriteExpect};

use crate::atom::*;
use crate::initiate::*;
use crate::maths;
use crate::constant;

/// # Euler Integration
/// 
/// The EulerIntegrationSystem integrates the classical equations of motion for particles using the euler method:
/// ```  
/// x = x + v * dt
/// v = v + F/m*dt
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
			vel.vel = maths::array_addition(&vel.vel,&maths::array_multiply(&force.force,1./(constant::AMU*mass.value)*t.t));
			pos.pos = maths::array_addition(&pos.pos,&maths::array_multiply(&vel.vel,t.t));
		}
	}
}