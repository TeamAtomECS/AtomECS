//! Handles indexes for laser beams.
//!

use bevy::prelude::*;

/// An index that uniquely identifies a laser entity.
/// The index value corresponds to the position of each laser in per-atom sampler arrays.
///
/// Default [LaserIndex]s are created with `initiated: false`.
/// Once the index is set, initiated is set to true.
/// This is used to detect if all lasers in the simulation are correctly indexed, in case new lasers are added.
#[derive(Clone, Copy, Default, Component)]
#[component(storage = "SparseSet")]
pub struct LaserIndex {
    pub index: usize,
    pub initiated: bool,
}

/// Assigns a unique [LaserIndex] to each laser.
pub fn index_lasers(mut query: Query<&mut LaserIndex>) {
    let mut iter = 0;
    let mut need_to_assign_indices = false;
    for index in query.iter() {
        if !index.initiated {
            need_to_assign_indices = true;
        }
    }
    if need_to_assign_indices {
        for mut index in query.iter_mut() {
            index.index = iter;
            index.initiated = true;
            iter += 1;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_index_lasers() {
        let mut app = App::new();

        let test_entity_1 = app.world_mut().spawn(LaserIndex::default()).id();
        let test_entity_2 = app.world_mut().spawn(LaserIndex::default()).id();

        app.add_systems(Update, index_lasers);
        app.update();

        let index_1 = app
            .world()
            .entity(test_entity_1)
            .get::<LaserIndex>()
            .expect("entity not found");
        let index_2 = app
            .world()
            .entity(test_entity_2)
            .get::<LaserIndex>()
            .expect("entity not found");
        assert_ne!(index_1.index, index_2.index);
    }
}
