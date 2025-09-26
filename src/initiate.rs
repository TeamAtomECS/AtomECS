//! A module responsible for initiating newly created atoms.
//!
//! When new atoms are added to the simulation, a [NewlyCreated](struct.NewlyCreated.html)
//! component is attached. This provides a signal that modules can use to identify which
//! atoms are new, and thus to attach any required components. For instance, the `magnetics`
//! module attaches a [MagneticFieldSampler](struct.MagneticFieldSampler.html) to new atoms
//! so that the `magnetics` systems can calculate fields at each atom's location.
//!
//! This module defines the [NewlyCreated](struct.NewlyCreated.html) component, and also the
//! system responsible for cleaning up these components each integration step.

use bevy::prelude::*;

/// A marker component that indicates an entity has been created within the last frame.
///
/// The main use of this component is to allow different modules to identify when an atom has been created and to attach any appropriate components required.
/// For instance, a [NewlyCreated] atom could have a field sampler added to it so that magnetic systems will be able to calculate fields at the atom's position.
#[derive(Component, Default)]
pub struct NewlyCreated;

/// Removes [NewlyCreated] marker components from atoms.
///
/// The marker is originally added to atoms when they are first added to the simulation, which allows other Systems
/// to add any required components to atoms.
///
/// ## When does this system run?
///
/// This system runs *just before* new atoms are added to the world.
/// Thus, any atoms flagged as [NewlyCreated] from the previous frame are deflagged before the new flagged atoms are created.
fn deflag_new_atoms(mut commands: Commands, query: Query<Entity, With<NewlyCreated>>) {
    for ent in query.iter() {
        commands.entity(ent).remove::<NewlyCreated>();
    }
}

pub struct InitiatePlugin;
impl Plugin for InitiatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, deflag_new_atoms.before(crate::atom_sources::emit::emit_once_system));
    }
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;

    /// Test the [NewlyCreated] component is properly removed from atoms after an update.
    #[test]
    fn test_deflag_new_atoms_system() {
        let mut app = App::new();
        app.add_plugins(InitiatePlugin);

        let test_entity = app.world_mut().spawn(NewlyCreated).id();
        app.update();
        assert!(!app.world().entity(test_entity).contains::<NewlyCreated>());
    }
}
