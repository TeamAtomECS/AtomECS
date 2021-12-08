//! Stores atomic trajectories in memory.

use crate::atom::*;
use crate::integrator::Step;
use specs::{Component, Entities, Join, ReadExpect, ReadStorage, System};

/// A system that stores atomic trajectories in memory.
///
/// This system stores per-atom data `T` at defined intervals.
/// The data type `T` must be a [Component](specs::Component), and
/// implement the Clone trait.
///
/// This system is only intended as a lightweight form of output for simple
/// examples. It is *not* intended for serious use for a number of reasons:
///  * Large numbers of atoms become unfeasible to store in memory.
///  * This output system offers no way to sort or identify atoms.
///  * Storing data in stretchy arrays is inefficient.
///
/// A better alternative is to use the [FileOutputSystem](crate::output::file_output::FileOutputSystem).
pub struct MemoryOutputSystem<T: Component + Clone> {
    /// The [FileOutputSystem](struct.FileOutputSystem.html) writes to file every time
    /// this number of steps are completed.
    pub interval: u64,

    /// Data stored in the file output system.
    payload: Vec<Vec<T>>,
}

impl<T> MemoryOutputSystem<T>
where
    T: Component + Clone,
{
    pub fn new(interval: u64) -> Self {
        MemoryOutputSystem {
            interval,
            payload: Vec::new(),
        }
    }
}

impl<'a, T> System<'a> for MemoryOutputSystem<T>
where
    T: Component + Clone,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, T>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, data, atoms, step): Self::SystemData) {
        if step.n % self.interval == 0 {
            // Lump the atom vector into memory.
            let mut vec = Vec::new();
            for (data, _, _) in (&data, &atoms, &entities).join() {
                vec.push(data.clone());
            }
            self.payload.push(vec);
        }
    }
}
