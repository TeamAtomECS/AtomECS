//! Module for performing linear ramps of quantities.
//!
//! Ramps are characterised by the values the component should take at different keyframes.
//! The component is then linearly interpolated between these values as the simulation proceeds.
//!
//! To ramp a component `T`'s values, add a `Ramp<T>` to the entity. You should also create a
//! `RampUpdateSystem<T>` and add it to the dispatcher.

use specs::{Component, HashMapStorage, Join, ReadExpect, System, WriteStorage};

use crate::integrator::{Step, Timestep};
use std::marker::PhantomData;

pub trait Lerp<T> {
    /// Linearly interpolates from self to b by the given amount (in range 0 to 1).
    fn lerp(&self, b: &T, amount: f64) -> Self;
}

pub struct Ramp<T>
where
    T: Lerp<T> + Component + Clone,
{
    /// Paired list of times and values to have at each time.
    pub keyframes: Vec<(f64, T)>,
    /// prev keyframe in the keyframe list.
    prev: usize,
}

impl<T> Ramp<T>
where
    T: Lerp<T> + Component + Clone,
{
    pub fn get_value(&mut self, current_time: f64) -> T {
        // check if we need to advance cursor
        let (t0, _) = &self.keyframes[self.prev];
        if current_time > *t0 {
            self.prev = (self.prev + 1).min(self.keyframes.len() - 1);
        }

        // if at end, just return last frame value.
        if self.prev == self.keyframes.len() - 1 {
            let (_, last) = &self.keyframes[self.prev];
            return last.clone();
        }

        // not on last element, lerp between
        let (t1, val_a) = &self.keyframes[self.prev];
        let (t2, val_b) = &self.keyframes[self.prev + 1];
        let amount = (current_time - t1) / (t2 - t1);
        return val_a.lerp(&val_b, amount);
    }
}

impl<T> Component for Ramp<T>
where
    T: Lerp<T> + Component + Sync + Send + Clone,
{
    type Storage = HashMapStorage<Self>;
}

pub struct RampUpdateSystem<T>
where
    T: Component,
    T: Lerp<T>,
{
    ramped: PhantomData<T>,
}

impl<'a, T> System<'a> for RampUpdateSystem<T>
where
    T: Lerp<T> + Component + Sync + Send + Clone,
{
    type SystemData = (
        WriteStorage<'a, T>,
        WriteStorage<'a, Ramp<T>>,
        ReadExpect<'a, Timestep>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (mut comps, mut ramps, timestep, step): Self::SystemData) {
        let current_time = step.n as f64 * timestep.delta;

        for (ramp, comp) in (&mut ramps, &mut comps).join() {
            comp.clone_from(&ramp.get_value(current_time));
        }
    }
}
