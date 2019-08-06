extern crate specs;
use crate::atom::Position;
use specs::{Entities, Join, ReadStorage, System};

/// This system is responsible for delete atoms that have strayed out of the simulation region.
pub struct DestroyAtomsSystem;

impl<'a> System<'a> for DestroyAtomsSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>);

    fn run(&mut self, (ents, position): Self::SystemData) {
        for (entity, position) in (&ents, &position).join() {
            if position.pos.norm_squared() > (0.5_f64).powf(2.0) {
                ents.delete(entity).expect("Could not delete entity");
            }
        }
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
    fn test_delete_atoms() {
        let mut test_world = World::new();
        test_world.register::<Position>();

        let test_entity1 = test_world.create_entity().with(Position::new()).build();
        let test_entity2 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(2.0, 2.0, 2.0),
            })
            .build();

        let mut system = DestroyAtomsSystem;
        system.run_now(&test_world.res);
        test_world.maintain();

        let positions = test_world.read_storage::<Position>();
        assert_eq!(positions.get(test_entity1).is_none(), false);
        assert_eq!(positions.get(test_entity2).is_none(), true);
    }
}
