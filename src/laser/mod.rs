/// The laser module relates to optical scattering forces.
pub mod cooling;
pub mod force;
pub mod gaussian;
pub mod intensity;

extern crate specs;
use specs::{
	Component, DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage,
	System, VecStorage, World, WriteStorage,
};
use crate::initiate::NewlyCreated;

/// Attachs components used for optical force calculation to newly created atoms.
///
/// This system attaches the `RandKick` and `InteractionLaserALL` components to `NewlyCreated` entities.
/// Both components are required by other laser `System`s to perform calculations of optical scattering forces.
pub struct AttachLaserComponentsToNewlyCreatedAtomsSystem;

impl<'a> System<'a> for AttachLaserComponentsToNewlyCreatedAtomsSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, NewlyCreated>,
		Read<'a, LazyUpdate>,
	);

	fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
		for (ent, _) in (&ent, &newly_created).join() {
			//updater.insert(ent, CoolingForce::default());
		}
	}
}