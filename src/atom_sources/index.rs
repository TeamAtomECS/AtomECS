use crate::atom::*;

use specs::{
	Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System,
	WriteExpect, 
};

/// Resource that keeps track of the number of atoms created in the simulation.
pub struct AtomNumber {
    /// The next free index available.
    pub next: u32
}

/// This system assigns [AtomIndex](struct.AtomIndex.html)s to [Atoms](struct.Atom.html) which do not have them.
pub struct AssignAtomIndexSystem;
impl<'a> System<'a> for AssignAtomIndexSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomIndex>,
        WriteExpect<'a, AtomNumber>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (entities, atoms, indices, number, updater): Self::SystemData) {
        
        let mut n = number.next;
        for (ent, _, ()) in (&entities, &atoms, !&indices).join() {
            updater.insert(ent, AtomIndex { index: n });
        }
    }
}