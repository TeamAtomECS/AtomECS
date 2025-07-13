//! Systems and Components used to delete atoms from the simulation.
//!
//! Atoms that leave a region defined by the [SimulationBounds](struct.SimulationBounds.html)
//! world resource will be deleted by the [DestroyOutOfBoundAtomsSystem](struct.DestroyOutOfBoundAtomsSystem.html).
//! Removing atoms that will not be of interest for further simulation (eg, those that escape the trapping region)
//! ensures that CPU time will not be wasted simulating them.

use bevy::prelude::*;

/// A system that deletes entities which have been marked for destruction using the [ToBeDestroyed] component.
fn delete_to_be_destroyed_entities(
    query: Query<Entity, With<ToBeDestroyed>>,
    mut commands: Commands,
) {
    query.into_iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

/// This plugin implements removal of atoms marked as [ToBeDestroyed].
///
/// See also [crate::destructor].
pub struct DestroyAtomsPlugin;
impl Plugin for DestroyAtomsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, delete_to_be_destroyed_entities);
    }
}

/// [Component](struct.Component.html) that marks an entity to be removed from the simulation by the [DestroyOutOfBoundAtomsSystem](struct.DestroyOutOfBoundAtomsSystem.html).
///
/// Note that the entity will not be removed instantly once the component is added, as it requires a call to `world.maintain`.
/// The entity will typically persist for the remainder of the frame.
#[derive(Default, Component)]
pub struct ToBeDestroyed;

pub mod tests {
    // These imports are actually needed! The compiler is getting confused and warning they are not.
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_imports)]
    use crate::atom::Position;

    #[test]
    fn test_to_be_destroyed_system() {
        let mut app = App::new();
        app.add_plugins(DestroyAtomsPlugin);

        let test_entity1 = app.world_mut().spawn(Position::default()).id();
        let test_entity2 = app
            .world_mut()
            .spawn(Position::default())
            .insert(ToBeDestroyed)
            .id();
        app.update();
        assert!(app.world().get_entity(test_entity1).is_ok());
        assert!(app.world().get_entity(test_entity2).is_err());
    }
}
