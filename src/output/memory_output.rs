//! Stores atomic trajectories in memory.

use crate::atom::*;
use crate::integrator::Step;

use bevy::prelude::*;

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
#[derive(Resource)]
pub struct MemoryOutputResource<T: Component + Clone> {
    /// The [FileOutputSystem](struct.FileOutputSystem.html) writes to file every time
    /// this number of steps are completed.
    pub interval: u64,

    /// Data stored in the file output system.
    payload: Vec<Vec<T>>,
}

impl<T> MemoryOutputResource<T>
where
    T: Component + Clone,
{
    pub fn new(interval: u64) -> Self {
        MemoryOutputResource {
            interval,
            payload: Vec::new(),
        }
    }
}

pub fn save_to_memory<T>(
    step: Res<Step>,
    mut memory_resource: ResMut<MemoryOutputResource<T>>,
    query: Query<&T, With<Atom>>,
) where
    T: Component + Clone,
{
    if step.n % memory_resource.interval == 0 {
        let mut vec = Vec::new();
        for data in query.iter() {
            vec.push(data.clone());
        }
        memory_resource.payload.push(vec);
    }
}
