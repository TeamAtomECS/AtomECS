extern crate specs;
use crate::atom::{Atom, Position};
use specs::{Component, Entities, ReadExpect,HashMapStorage, Join, NullStorage, ReadStorage, System};
extern crate nalgebra;
use nalgebra::Vector3;
/// Deletes entities which have been marked for destruction.
pub struct DeleteToBeDestroyedEntitiesSystem;
impl<'a> System<'a> for DeleteToBeDestroyedEntitiesSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, ToBeDestroyed>);

    fn run(&mut self, (ents, _des): Self::SystemData) {
        for (entity, _des) in (&ents, &_des).join() {
            ents.delete(entity).expect("Could not delete entity");
        }
    }
}

pub struct BoundaryMarker{
    pub value: bool,
}



/// Deletes atoms that have strayed outside of the simulation region.
pub struct DestroyOutOfBoundAtomsSystem;
impl<'a> System<'a> for DestroyOutOfBoundAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, BoundaryMarker>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Atom>,
    );

    fn run(&mut self, (entities, boundary, positions, atoms): Self::SystemData) {
        if boundary.value {
            for (entity, position, _) in (&entities, &positions, &atoms).join() {

                if out_of_bound(&position.pos) {
                    entities.delete(entity).expect("Could not delete entity");
                }
            }
        }
    }
}

fn out_of_bound(position: &Vector3<f64>) -> bool {
    let mut result = true;
    if position.norm() < 0.060 && position[0] > -0.025 && position[0] < 0.025 {
        result = false
    }
    if position[2].powf(2.0) + position[1].powf(2.0) < 0.02_f64.powf(2.0) {
        result = false
    }
    result
}

/// Component that marks an entity for deletion.
pub struct ToBeDestroyed;
impl Component for ToBeDestroyed {
    type Storage = NullStorage<Self>;
}
impl Default for ToBeDestroyed {
    fn default() -> Self {
        ToBeDestroyed {}
    }
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
    use nalgebra::Vector3;

    #[test]
    fn test_destroy_out_of_bounds_system() {
        let mut test_world = World::new();
        test_world.register::<Position>();
        test_world.register::<Atom>();

        test_world.register::<BoundaryMarker>();
        test_world.create_entity().with(BoundaryMarker {}).build();
        let test_entity1 = test_world
            .create_entity()
            .with(Position::new())
            .with(Atom::default())
            .build();
        let test_entity2 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(2.0, 2.0, 2.0),
            })
            .with(Atom::default())
            .build();

        let mut system = DestroyOutOfBoundAtomsSystem;
        system.run_now(&test_world.res);
        test_world.maintain();

        let positions = test_world.read_storage::<Position>();
        assert_eq!(positions.get(test_entity1).is_none(), false);
        assert_eq!(positions.get(test_entity2).is_none(), true);
    }

    #[test]
    fn test_to_be_destroyed_system() {
        let mut test_world = World::new();
        test_world.register::<Position>();
        test_world.register::<ToBeDestroyed>();
        let test_entity1 = test_world.create_entity().with(Position::new()).build();
        let test_entity2 = test_world
            .create_entity()
            .with(Position::new())
            .with(ToBeDestroyed)
            .build();

        let mut system = DeleteToBeDestroyedEntitiesSystem;
        system.run_now(&test_world.res);
        test_world.maintain();

        let positions = test_world.read_storage::<Position>();
        assert_eq!(positions.get(test_entity1).is_none(), false);
        assert_eq!(positions.get(test_entity2).is_none(), true);
    }
}
