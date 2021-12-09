//! Handles indexes for laser beams.
//!

use specs::prelude::*;

/// An index that uniquely identifies a laser entity.
/// The index value corresponds to the position of each laser in per-atom sampler arrays.
///
/// Default `LaserIndex`s are created with `initiated: false`.
/// Once the index is set, initiated is set to true.
/// This is used to detect if all lasers in the simulation are correctly indexed, in case new lasers are added.
#[derive(Clone, Copy, Default)]
pub struct LaserIndex {
    pub index: usize,
    pub initiated: bool,
}
impl Component for LaserIndex {
    type Storage = HashMapStorage<Self>;
}
/// Assigns unique indices to laser entities.
pub struct IndexLasersSystem;
impl<'a> System<'a> for IndexLasersSystem {
    type SystemData = WriteStorage<'a, LaserIndex>;

    fn run(&mut self, mut indices: Self::SystemData) {
        let mut iter = 0;
        let mut need_to_assign_indices = false;
        for index in (&indices).join() {
            if !index.initiated {
                need_to_assign_indices = true;
            }
        }
        if need_to_assign_indices {
            for mut index in (&mut indices).join() {
                index.index = iter;
                index.initiated = true;
                iter += 1;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_index_lasers() {
        let mut test_world = World::new();
        test_world.register::<LaserIndex>();

        let test_entity_1 = test_world
            .create_entity()
            .with(LaserIndex::default())
            .build();
        let test_entity_2 = test_world
            .create_entity()
            .with(LaserIndex::default())
            .build();

        let mut system = IndexLasersSystem;
        system.run_now(&test_world);
        test_world.maintain();

        let storage = test_world.read_storage::<LaserIndex>();
        let index_1 = storage.get(test_entity_1).expect("entity not found");
        let index_2 = storage.get(test_entity_2).expect("entity not found");

        assert_ne!(index_1.index, index_2.index);
    }
}
