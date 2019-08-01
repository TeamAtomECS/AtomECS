/// The laser module relates to optical scattering forces.
pub mod cooling;
pub mod force;
pub mod gaussian;
pub mod intensity;

extern crate specs;
use crate::initiate::NewlyCreated;
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System};

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

	fn run(&mut self, (ent, newly_created, _updater): Self::SystemData) {
		for (_, _) in (&ent, &newly_created).join() {
			//updater.insert(ent, CoolingForce::default());
		}
	}
}
