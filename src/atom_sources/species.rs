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

pub trait Species : AtomCreator {}
impl<T> Species for T where T : AtomCreator {}

/// Generates a species struct that can be used in an atom source.
/// 
/// # Arguments:
/// * `species_name`: name of the generated struct.
/// * `transition`: laser cooling transition to use.
/// * `mass`: mass of this species in atomic mass units.
macro_rules! species {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    ($species_name:ident, $transition: ident, $mass: literal) => {
        /// A species that can be used in an atom source.
        #[derive(Copy, Clone, Default)]
        pub struct $species_name;
        impl AtomCreationModifier for $species_name {
            fn mutate(updater: &LazyUpdate, new_atom: Entity) {
                updater.insert(new_atom, $transition::default());
            }
        }
    };
}

species!(Strontium88, Strontium88_461, 88);
species!(Rubidium87, Strontium88_461, 87);
