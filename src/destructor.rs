//! Systems and Components used to delete atoms from the simulation.
//!
//! Atoms that leave a region defined by the [SimulationBounds](struct.SimulationBounds.html)
//! world resource will be deleted by the [DestroyOutOfBoundAtomsSystem](struct.DestroyOutOfBoundAtomsSystem.html).
//! Removing atoms that will not be of interest for further simulation (eg, those that escape the trapping region)
//! ensures that CPU time will not be wasted simulating them.
extern crate specs;
use specs::prelude::*;

use crate::{simulation::Plugin, integrator::{INTEGRATE_POSITION_SYSTEM_NAME}};

/// A system that deletes entities which have been marked for destruction using the [ToBeDestroyed](struct.ToBeDestroyed.html) component.
pub struct DeleteToBeDestroyedEntitiesSystem;
impl<'a> System<'a> for DeleteToBeDestroyedEntitiesSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, ToBeDestroyed>);

    fn run(&mut self, (ents, des): Self::SystemData) {
        for (entity, _) in (&ents, &des).join() {
            ents.delete(entity).expect("Could not delete entity");
        }
    }
}

/// This plugin implements removal of atoms marked as `ToBeDestroyed`.
/// 
/// See also [crate::destructor].
pub struct DestroyAtomsPlugin;
impl Plugin for DestroyAtomsPlugin {
    fn build(&self, builder: &mut crate::simulation::SimulationBuilder) {
        builder.dispatcher_builder.add(
            DeleteToBeDestroyedEntitiesSystem,
            "",
            &[INTEGRATE_POSITION_SYSTEM_NAME],
        );
    }
    fn deps(&self) -> Vec::<Box<dyn Plugin>> {
        Vec::new()
    }
}

/// [Component](struct.Component.html) that marks an entity to be removed from the simulation by the [DestroyOutOfBoundAtomsSystem](struct.DestroyOutOfBoundAtomsSystem.html).
///
/// Note that the entity will not be removed instantly once the component is added, as it requires a call to `world.maintain`.
/// The entity will typically persist for the remainder of the frame.
#[derive(Default)]
pub struct ToBeDestroyed;
impl Component for ToBeDestroyed {
    type Storage = NullStorage<Self>;
}

pub mod tests {
    // These imports are actually needed! The compiler is getting confused and warning they are not.
    #[allow(unused_imports)]
    use super::*;
    extern crate specs;
    #[allow(unused_imports)]
    use specs::{Builder, Entity, RunNow, World};
    extern crate nalgebra;
    #[allow(unused_imports)]
    use crate::atom::Position;
    #[allow(unused_imports)]
    use nalgebra::Vector3;

    #[test]
    fn test_to_be_destroyed_system() {
        let mut test_world = World::new();
        test_world.register::<ToBeDestroyed>();
        test_world.register::<Position>();
        let test_entity1 = test_world.create_entity().with(Position::new()).build();
        let test_entity2 = test_world
            .create_entity()
            .with(Position::new())
            .with(ToBeDestroyed)
            .build();

        let mut system = DeleteToBeDestroyedEntitiesSystem;
        system.run_now(&test_world);
        test_world.maintain();

        let positions = test_world.read_storage::<Position>();
        assert!(positions.get(test_entity1).is_some());
        assert!(positions.get(test_entity2).is_none());
    }
}
