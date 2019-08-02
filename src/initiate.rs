extern crate specs;
use specs::{Component,VecStorage,NullStorage,Entities,Join,LazyUpdate,Read,ReadStorage,System};
use crate::constant::{C,BOHRMAG};

pub mod atom_create;
//pub mod ecs;

pub struct AtomInfo{
	/// 
	pub mup:f64,
	pub mum:f64,
	pub muz:f64,
	/// Frequency of the laser cooling transition, Hz.
	pub frequency:f64,
	/// Linewidth of the laser cooling transition, Hz
	pub gamma:f64,
	
	/// Saturation intensity, in units of W/m^2.
	pub saturation_intensity:f64
}

impl Component for AtomInfo{
	type Storage = VecStorage<Self>;	
}
impl AtomInfo {
	/// Creates an `AtomInfo` component populated with parameters for Rubidium. 
	/// The parameters are taken from Daniel Steck's Data sheet on Rubidium-87.
	pub fn rubidium() -> Self { 
		AtomInfo { mup: BOHRMAG,
		mum: -BOHRMAG,
		muz: 0.0,
		frequency: C / 780.0e-9,
		gamma: 6.065e6, // [Steck, Rubidium87]
		saturation_intensity: 16.69 // [Steck, Rubidium 87, D2 cycling transition]
		}
	}
}


/// A marker component that indicates an entity has been `NewlyCreated`. 
/// The main use of this component is to allow different modules to identify when an atom has been created and to attach any appropriate components required.
/// For instance, a NewlyCreated atom could have a field sampler added to it so that magnetic systems will be able to calculate fields at the atom's position.
#[derive(Component)]
#[storage(NullStorage)]
pub struct NewlyCreated;

impl Default for NewlyCreated {
	fn default() -> Self { NewlyCreated{} }
}

/// This system is responsible for removing the `NewlyCreated` marker component from atoms.
/// 
/// The marker is originally added to atoms when they are first added to the simulation, which allows other Systems
/// to add any required components to atoms.
/// 
/// ## When should this system run?
/// 
/// This system runs *before* new atoms are added to the world.
/// Thus, any atoms flagged as `NewlyCreated` from the previous frame are deflagged before the new flagged atoms are created.
/// Be careful of properly maintaining the world at the correct time;
/// LazyUpdate is used, so changes to remove the `NewlyCreated` components will only be enacted after the call to `world.maintain()`.
pub struct DeflagNewAtomsSystem;

impl<'a> System<'a> for DeflagNewAtomsSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a,NewlyCreated>,
		Read<'a, LazyUpdate>,
	);

	fn run(&mut self, (ent,newly_created, updater): Self::SystemData) {
		for (ent,_newly_created) in (&ent, &newly_created).join() {
			updater.remove::<NewlyCreated>(ent);
		}
	}
}

pub mod tests {
	// These imports are actually needed! The compiler is getting confused and warning they are not.
	#[allow(unused_imports)]
	use super::*;
	extern crate specs;
	#[allow(unused_imports)]
	use specs::{Builder, DispatcherBuilder, World};

	/// Tests that the NewlyCreated component is properly removed from atoms via the DeflagNewAtomsSystem.
	#[test]
	fn test_deflag_new_atoms_system() {
		let mut test_world = World::new();

		let mut dispatcher = DispatcherBuilder::new()
			.with(DeflagNewAtomsSystem, "deflagger", &[])
			.build();
		dispatcher.setup(&mut test_world.res);

		let test_entity = test_world
			.create_entity()
			.with(NewlyCreated)
			.build();

		dispatcher.dispatch(&mut test_world.res);
		test_world.maintain();

		let created_flags = test_world.read_storage::<NewlyCreated>();
		assert_eq!(created_flags.contains(test_entity), false);
	}
}