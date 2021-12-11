//! Module for performing linear ramps of quantities.
//!
//! Ramps are characterised by the values the component should take at different keyframes.
//! The component is then linearly interpolated between these values as the simulation proceeds.
//!
//! To ramp a component `T`'s values, add a `Ramp<T>` to the entity. You should also create a
//! `RampUpdateSystem<T>` and add it to the dispatcher.
//!
//! Only components which implement the `Lerp` trait can be ramped.
//! You can either explicitly implement this trait for your types, or use `[#derive(Clone,Lerp)]`.
//! The derive implementation is crude, and assumes:
//!   * The struct implements `Clone`.
//!   * The fields can all be multiplied by an f64 and added (eg `f64` and `Vector3<f64>` types).

use specs::prelude::*;

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
        if !self.at_end() {
            let (t0, _) = &self.keyframes[self.prev + 1];
            if current_time > *t0 {
                self.prev = (self.prev + 1).min(self.keyframes.len() - 1);
            }
        }
        // if at end, return last frame value.
        if self.at_end() {
            let (_, last) = &self.keyframes[self.prev];
            return last.clone();
        }

        // not on last element, lerp between
        let (t1, val_a) = &self.keyframes[self.prev];
        let (t2, val_b) = &self.keyframes[self.prev + 1];
        let amount = (current_time - t1) / (t2 - t1);
        val_a.lerp(val_b, amount)
    }

    fn at_end(&self) -> bool {
        self.prev == self.keyframes.len() - 1
    }

    pub fn new(keyframes: Vec<(f64, T)>) -> Self {
        Ramp {
            keyframes,
            prev: 0,
        }
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

impl<T> Default for RampUpdateSystem<T>
where
    T: Component,
    T: Lerp<T>,
{
    fn default() -> Self {
        Self {
            ramped: PhantomData,
        }
    }
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

pub mod tests {
    use super::*;
    extern crate specs;
    use specs::{Component, HashMapStorage};

    #[derive(Clone, Lerp)]
    struct ALerpComp {
        value: f64,
    }

    impl Component for ALerpComp {
        type Storage = HashMapStorage<Self>;
    }

    #[test]
    fn test_ramp() {
        use assert_approx_eq::assert_approx_eq;

        let frames = vec![
            (0.0, ALerpComp { value: 0.0 }),
            (1.0, ALerpComp { value: 1.0 }),
            (2.0, ALerpComp { value: 0.0 })];
        let mut ramp = Ramp {
            prev: 0,
            keyframes: frames,
        };

        {
            let comp = ramp.get_value(0.0);
            assert_approx_eq!(comp.value, 0.0, std::f64::EPSILON);
        }
        {
            let comp = ramp.get_value(0.5);
            assert_approx_eq!(comp.value, 0.5, std::f64::EPSILON);
        }
        {
            let comp = ramp.get_value(1.0);
            assert_approx_eq!(comp.value, 1.0, std::f64::EPSILON);
        }
        {
            let comp = ramp.get_value(1.5);
            assert_approx_eq!(comp.value, 0.5, std::f64::EPSILON);
        }
        {
            let comp = ramp.get_value(2.0);
            assert_approx_eq!(comp.value, 0.0, std::f64::EPSILON);
        }
        {
            let comp = ramp.get_value(2.5);
            assert_approx_eq!(comp.value, 0.0, std::f64::EPSILON);
        }
    }

    #[test]
    fn test_ramp_system() {
        use crate::integrator::VelocityVerletIntegratePositionSystem;
        use assert_approx_eq::assert_approx_eq;
        use specs::{Builder, DispatcherBuilder, ReadStorage, World};

        let mut test_world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(VelocityVerletIntegratePositionSystem, "integrator", &[])
            .with(
                RampUpdateSystem::<ALerpComp>::default(),
                "update_lerp_comp",
                &["integrator"],
            )
            .build();
        dispatcher.setup(&mut test_world);

        let frames = vec![
            (0.0, ALerpComp { value: 0.0 }),
            (1.0, ALerpComp { value: 1.0 })];
        let ramp = Ramp {
            prev: 0,
            keyframes: frames,
        };

        let test_entity = test_world
            .create_entity()
            .with(ALerpComp { value: 0.0 })
            .with(ramp)
            .build();

        let dt = 0.1;
        test_world.insert(Timestep { delta: dt });
        test_world.insert(Step { n: 0 });

        // Perform dispatcher loop to ramp components.
        for i in 1..10 {
            dispatcher.dispatch(&test_world);

            let comps: ReadStorage<ALerpComp> = test_world.system_data();
            assert_approx_eq!(
                comps.get(test_entity).expect("Entity not found").value,
                i as f64 * dt,
                std::f64::EPSILON
            );
        }
    }
}
