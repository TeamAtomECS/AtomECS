//! Time-Orbiting Potential trap
//!

extern crate nalgebra;
extern crate specs;
use crate::constant::PI;
use crate::integrator::{Step, Timestep};
use crate::magnetic::MagneticFieldSampler;
use crate::ramp::Lerp;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage, Join, ReadExpect, ReadStorage, System, WriteStorage};

/// A component representing a Time-Orbiting Potential (TOP)
#[derive(Clone, Lerp)]
pub struct TimeOrbitingPotential {
    /// Amplitude of the field in T
    pub amplitude: f64,
    /// A unit vector pointing along the rotational axis of the TOP.
    pub direction: Vector3<f64>,
    ///Frequency of rotation in Hz
    pub frequency: f64,
}
impl TimeOrbitingPotential {
    //create a new TOP with amplitude in gauss & frequency in Hz
    pub fn gauss(amplitude: f64, frequency: f64) -> Self {
        Self {
            amplitude: amplitude * 1e-4,
            direction: Vector3::z(),
            frequency: frequency,
        }
    }
}
impl Component for TimeOrbitingPotential {
    type Storage = HashMapStorage<Self>;
}

pub struct TimeOrbitingPotentialSystem;
impl<'a> System<'a> for TimeOrbitingPotentialSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, TimeOrbitingPotential>,
        ReadExpect<'a, Timestep>,
        ReadExpect<'a, Step>,
    );
    fn run(&mut self, (mut samplers, tops, timestep, step): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for top in (&tops).join() {
            (&mut samplers).par_join().for_each(|sampler| {
                //TODO: allow rotation around arbitrary axis
                let time = timestep.delta * step.n as f64;
                let top_field = top.amplitude
                    * Vector3::new(
                        (2.0 * PI * top.frequency * time).cos(),
                        (2.0 * PI * top.frequency * time).sin(),
                        0.0,
                    );

                sampler.field = sampler.field + top_field;
            });
        }
    }
}
