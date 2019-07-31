extern crate specs;
use specs::{Component,VecStorage,NullStorage};

pub mod atom_create;

pub struct AtomInfo{
	pub mup:f64,
	pub mum:f64,
	pub muz:f64,
	pub mass:u64,
	pub frequency:f64,
	pub gamma:f64,
}

impl Component for AtomInfo{
	type Storage = VecStorage<Self>;	
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