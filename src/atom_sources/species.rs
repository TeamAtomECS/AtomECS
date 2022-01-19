//! Different implementations for created atom species.

use specs::prelude::*;

use crate::{laser_cooling::transition::Strontium88_461};

/// Allows atoms to be modified after they are created.
pub trait AtomCreationModifier {
    /// Modifies the created atom
    fn mutate(updater: &LazyUpdate, new_atom: Entity);
}
pub trait AtomCreator : AtomCreationModifier + Copy + Send + Sync + Default {}
impl<T> AtomCreator for T where T : AtomCreationModifier + Copy + Send + Sync + Default {}

#[derive(Copy, Clone, Default)]
pub struct Strontium87;
impl AtomCreationModifier for Strontium87 {
    fn mutate(updater: &LazyUpdate, new_atom: Entity) {
        // eventually add mass and species here too.
        //updater.insert(new_atom, Mass { value: 87. });
        updater.insert(new_atom, Strontium88_461::default());
    }
}