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

use bevy::prelude::*;
use bevy::ecs::component::Mutable;

use crate::integrator::{AtomECSBatchStrategy, Step, Timestep};
use std::marker::PhantomData;

pub trait Lerp<T> {
    /// Linearly interpolates from self to b by the given amount (in range 0 to 1).
    fn lerp(&self, b: &T, amount: f64) -> Self;
}

#[derive(Component)]
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
        Ramp { keyframes, prev: 0 }
    }
}

fn apply_ramp<T>(
    mut query: Query<(&mut T, &mut Ramp<T>)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
    step: Res<Step>,
) where
    T: Lerp<T> + Component<Mutability = Mutable> + Sync + Send + Clone,
{
    let current_time = step.n as f64 * timestep.delta;
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut comp, mut ramp)| {
            comp.clone_from(&ramp.get_value(current_time));
        });
}

/// Implements ramping of a given component type.
pub struct RampPlugin<T>
where
    T: Lerp<T> + Component + Clone,
{
    phantom: PhantomData<T>,
}
impl<T> Default for RampPlugin<T>
where
    T: Lerp<T> + Component + Sync + Send + Clone,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> Plugin for RampPlugin<T>
where
    T: Lerp<T> + Component<Mutability = Mutable> + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_ramp::<T>);
    }
}

pub mod tests {
    use super::*;

    #[derive(Clone, Lerp, Component)]
    struct ALerpComp {
        value: f64,
    }

    #[test]
    fn test_ramp() {
        use assert_approx_eq::assert_approx_eq;

        let frames = vec![
            (0.0, ALerpComp { value: 0.0 }),
            (1.0, ALerpComp { value: 1.0 }),
            (2.0, ALerpComp { value: 0.0 }),
        ];
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
        use assert_approx_eq::assert_approx_eq;

        let mut app = App::new();
        app.add_plugins(RampPlugin::<ALerpComp>::default());
        app.add_plugins(crate::integrator::IntegrationPlugin);

        let frames = vec![
            (0.0, ALerpComp { value: 0.0 }),
            (1.0, ALerpComp { value: 1.0 }),
        ];
        let ramp = Ramp {
            prev: 0,
            keyframes: frames,
        };

        let test_entity = app.world_mut().spawn(ALerpComp { value: 0.0 }).insert(ramp).id();

        let dt = 0.1;
        app.world_mut().insert_resource(Timestep { delta: dt });
        app.world_mut().insert_resource(Step { n: 0 });

        // Perform dispatcher loop to ramp components.
        for i in 1..10 {
            app.update();

            assert_approx_eq!(
                app.world()
                    .entity(test_entity)
                    .get::<ALerpComp>()
                    .expect("could not get component.")
                    .value,
                i as f64 * dt,
                std::f64::EPSILON
            );
        }
    }
}
