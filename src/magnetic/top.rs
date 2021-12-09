//! Time-Orbiting Potential trap
//! A rotating uniform bias field that creates an axially symmetric approximately harmonic trap when combined with another magnetic field such as a quadrupole
//! and time-averaged. The rotation frequency of the TOP should be much more than the oscillation frequency of the atoms, and much less than the Larmor frequency
//! of the atoms to avoid non-adiabatic loss (not modelled).
//! For more detail see e.g. W. Petrich, M. Anderson, J. Ensher, E. Cornell PRL 74, 3352, doi: https://doi.org/10.1103/PhysRevLett.74.3352

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
    ///Frequency of rotation in Hz
    pub frequency: f64,
}
impl TimeOrbitingPotential {
    //create a new TOP with amplitude in gauss & frequency in Hz
    pub fn gauss(amplitude: f64, frequency: f64) -> Self {
        Self {
            amplitude: amplitude * 1e-4,
            frequency,
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
                let time = timestep.delta * step.n as f64;
                let top_field = top.amplitude
                    * Vector3::new(
                        (2.0 * PI * top.frequency * time).cos(),
                        (2.0 * PI * top.frequency * time).sin(),
                        0.0,
                    );

                sampler.field += top_field;
            });
        }
    }
}
