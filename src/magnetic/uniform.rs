//! Uniform magnetic fields
use bevy::prelude::*;
use super::analytic::AnalyticField;
use crate::ramp::Lerp;
use nalgebra::Vector3;

/// A component representing a uniform bias field, of the form `B = [ B_x, B_y, B_z ]`
#[derive(Clone, Copy, Component, Lerp)]
#[component(storage = "SparseSet")]
pub struct UniformMagneticField {
    /// Vector field components with respect to the x,y,z cartesian axes, in units of Tesla.
    pub field: Vector3<f64>,
}
impl AnalyticField for UniformMagneticField {
    fn get_field(&self, _origin: Vector3<f64>, _field_point: Vector3<f64>) -> Vector3<f64> {
        self.field
    }

    fn calculate_jacobian(&self) -> bool {
        false // no point - zero everywhere.
    }
}

impl UniformMagneticField {
    /// Create a UniformMagneticField with components specified in units of Gauss.
    pub fn gauss(components: Vector3<f64>) -> UniformMagneticField {
        UniformMagneticField {
            field: components * 1.0e-4,
        }
    }

    /// Create a UniformMagneticField with components specified in units of Tesla.
    pub fn tesla(components: Vector3<f64>) -> UniformMagneticField {
        UniformMagneticField { field: components }
    }
}