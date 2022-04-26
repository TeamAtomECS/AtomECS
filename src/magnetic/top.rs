//! Time-Orbiting Potential trap
//! A rotating uniform bias field that creates an axially symmetric approximately harmonic trap when combined with another magnetic field such as a quadrupole
//! and time-averaged. The rotation frequency of the TOP should be much more than the oscillation frequency of the atoms, and much less than the Larmor frequency
//! of the atoms to avoid non-adiabatic loss (not modelled).
//! For more detail see e.g. W. Petrich, M. Anderson, J. Ensher, E. Cornell PRL 74, 3352, doi: https://doi.org/10.1103/PhysRevLett.74.3352

use crate::constant::PI;
use crate::integrator::{Step, Timestep};
use crate::ramp::Lerp;
use nalgebra::Vector3;
use bevy::prelude::*;
use super::uniform::UniformMagneticField;

/// The rotating linear field used for the Time-Orbiting Potential (TOP)
#[derive(Clone, Lerp, Component)]
#[component(storage = "SparseSet")]
pub struct UniformFieldRotator {
    /// Amplitude of the field in T
    pub amplitude: f64,
    ///Frequency of rotation in Hz
    pub frequency: f64,
}

pub fn rotate_uniform_fields(
    mut query: Query<(&UniformFieldRotator, &mut UniformMagneticField)>,
    timestep: Res<Timestep>,
    step: Res<Step>
) {
    let time = timestep.delta * step.n as f64;
    for (rotator, mut field) in query.iter_mut() {
        let top_field = rotator.amplitude
        * Vector3::new(
            (2.0 * PI * rotator.frequency * time).cos(),
            (2.0 * PI * rotator.frequency * time).sin(),
            0.0,
        );
        field.field = top_field;
    }
}