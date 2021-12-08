//! A module responsible for initiating newly created atoms.
//!
//! When new atoms are added to the simulation, a [NewlyCreated](struct.NewlyCreated.html)
//! component is attached. This provides a signal that modules can use to identify which
//! atoms are new, and thus to attach any required components. For instance, the `magnetics`
//! module attaches a [MagneticFieldSampler](struct.MagneticFieldSampler.html) to new atoms
//! so that the `magnetics` systems can calculate fields at each atom's location.
//!
//! This module defines the [NewlyCreated](struct.NewlyCreated.html) component, and also the
//! [DeflagNewAtomsSystem](struct.DeflagNewAtomsSystem.html) which is responsible for cleaning
//! up these components each integration step.

use specs::prelude::*;

/// A marker component that indicates an entity has been `NewlyCreated`.
/// The main use of this component is to allow different modules to identify when an atom has been created and to attach any appropriate components required.
/// For instance, a NewlyCreated atom could have a field sampler added to it so that magnetic systems will be able to calculate fields at the atom's position.
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct NewlyCreated;

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
        ReadStorage<'a, NewlyCreated>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
        for (ent, _newly_created) in (&ent, &newly_created).join() {
            updater.remove::<NewlyCreated>(ent);
        }
    }
}

pub mod tests {
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
        dispatcher.setup(&mut test_world);

        let test_entity = test_world.create_entity().with(NewlyCreated).build();

        dispatcher.dispatch(&test_world);
        test_world.maintain();

        let created_flags = test_world.read_storage::<NewlyCreated>();
        assert!(!created_flags.contains(test_entity));
    }
}
