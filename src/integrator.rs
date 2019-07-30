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
/// `x' = x + v * dt`.
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
			euler_updating(&mut vel,&mut pos,&force,&mass,t.t);
		}
	}
}

fn euler_updating(vel:&mut Velocity,pos:&mut Position,force:&Force,mass:&Mass,time:f64){
		vel.vel = maths::array_addition(&vel.vel,&maths::array_multiply(&force.force,1./(constant::AMU*mass.value)*time));
		pos.pos = maths::array_addition(&pos.pos,&maths::array_multiply(&vel.vel,time));
}

pub mod tests {
	// These imports are actually needed! The compiler is getting confused and warning they are not.
	#[allow(unused_imports)]
	use super::*;
	extern crate specs;
	#[allow(unused_imports)]
	use specs::{World,DispatcherBuilder,Builder};

	#[test]
	fn test_euler() {
		let mut pos = Position{pos:[1.,1.,1.]};
		let mut vel = Velocity{vel:[0.,1.,0.]};
		let time = 1.;
		let mass = Mass{value:1./constant::AMU};
		let force = Force{force:[0.,0.,1.]};
		euler_updating(&mut vel,&mut pos,&force,&mass,time);
		assert_eq!(vel.vel, [0.,1.,1.]);
		assert_eq!(pos.pos,[1.,2.,2.]);
	}

	/// Tests the [EulerIntegrationSystem] by creating a mock world and integrating the trajectory of one entity.
	#[test]
	fn test_euler_system()
	{
		let mut test_world = World::new();

		let mut dispatcher=DispatcherBuilder::new()
		.with(EulerIntegrationSystem, "integrator", &[])
		.build();
		
		dispatcher.setup(&mut test_world.res);

		let initial_position = [ 0.0, 0.1, 0.0];
		let initial_velocity = [ 1.0, 1.5, 0.4];
		let initial_force = [ 0.4, 0.5, -0.4];
		let mass = 2.0/constant::AMU;
		let test_entity = test_world.create_entity()
		.with(Position{pos:initial_position})
		.with(Velocity{vel:initial_velocity})
		.with(Force{force:initial_force})
		.with(Mass{value:mass})
		.build();

		let dt = 1.0;
		test_world.add_resource(Timestep{t:1.0});
		test_world.add_resource(Step{n:0});

		dispatcher.dispatch(&mut test_world.res);

		let velocities = test_world.read_storage::<Velocity>();
		let velocity = velocities.get(test_entity).expect("entity not found");
		assert_eq!(velocity.vel,maths::array_addition(&initial_velocity,&maths::array_multiply(&initial_force,&dt/2.)));
		let positions = test_world.read_storage::<Position>();
		let position = positions.get(test_entity).expect("entity not found");
		assert_eq!(position.pos,maths::array_addition(&initial_position, &maths::array_multiply(&velocity.vel, dt)));
	
	}

}