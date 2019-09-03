extern crate specs;
use crate::atom::{Atom, Position};
use specs::{Component, Entities, Join, NullStorage, Read, ReadStorage, System};
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

/// The [SimulationBounds](struct.SimulationBounds.html) is a resource that defines a bounding box encompassing the simulation region.
/// Atoms outside of the bounding box are deleted by the [DestroyOutOfBoundsAtomsSystem](struct.DestroyOutOfBoundAtomsSystem.html).
///
/// The simulation region is cubic. Atoms are deleted if any `abs(pos[i]) > half_width[i]`.
/// See [Position](struct.Position.html) for details of atom positions.
pub struct SimulationBounds {
    /// A vector defining the extent of the simulation region.
    pub half_width: Vector3<f64>,
}

/// A system that deletes atoms that have strayed outside of the simulation region.
pub struct DestroyOutOfBoundAtomsSystem;
impl<'a> System<'a> for DestroyOutOfBoundAtomsSystem {
    type SystemData = (
        Entities<'a>,
        Option<Read<'a, SimulationBounds>>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Atom>,
    );

    fn run(&mut self, (entities, boundary_option, positions, atoms): Self::SystemData) {
        match boundary_option {
            None => return,
            Some(boundary) => {
                for (entity, position, _) in (&entities, &positions, &atoms).join() {
                    if position.pos[0].abs() > boundary.half_width[0].abs()
                        || position.pos[1].abs() > boundary.half_width[1].abs()
                        || position.pos[2].abs() > boundary.half_width[2].abs()
                    {
                        entities.delete(entity).expect("Could not delete entity");
                    }
                }
            }
        }
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
    use nalgebra::Vector3;

    #[test]
    fn test_destroy_out_of_bounds_system() {
        let mut test_world = World::new();
        test_world.register::<Position>();
        test_world.register::<Atom>();

        test_world.add_resource(SimulationBounds {
            half_width: Vector3::new(1.0, 1.0, 1.0),
        });
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
    fn test_destroy_out_of_bounds_system_optional() {
        let mut test_world = World::new();
        test_world.register::<Position>();
        test_world.register::<Atom>();
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
        assert_eq!(positions.get(test_entity2).is_none(), false);
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
