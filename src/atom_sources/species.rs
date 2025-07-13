//! Different implementations for created atom species.

use bevy::prelude::*;
// use bevy::ecs::entity::Entity;

/// Allows atoms to be modified after they are created.
pub trait AtomCreationModifier {
    /// Modifies the created atom
    fn mutate(commands: &mut Commands, new_atom: Entity);
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
#[macro_export]
macro_rules! species {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    ($species_name:ident, $transition: ident, $mass: literal) => {
        /// A species that can be used in an atom source.
        #[derive(Copy, Clone, Default)]
        pub struct $species_name;
        impl $crate::atom_sources::species::AtomCreationModifier for $species_name {
            fn mutate(commands: &mut bevy::prelude::Commands, new_atom: bevy::ecs::entity::Entity) {
                commands.entity(new_atom).insert($transition::default());
            }
        }
    };
}
