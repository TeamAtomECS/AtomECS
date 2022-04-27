//! Reference frame orthogonal to the beam

use bevy::prelude::*;
use nalgebra::Vector3;

/// A component that stores the orthonormal basis vectors of a reference frame orthogonal to the beam.
#[derive(Clone, Copy, Component)]
#[component(storage = "SparseSet")]
pub struct Frame {
    pub x_vector: Vector3<f64>,
    pub y_vector: Vector3<f64>,
}

impl Frame {
    pub fn from_direction(beam_direction: Vector3<f64>, x_vector: Vector3<f64>) -> Self {
        let scalar_product: f64 = Vector3::dot(&beam_direction, &x_vector);
        if scalar_product != 0.0 {
            panic!("You entered non-orthogonal vectors!");
        }
        if beam_direction.norm() * x_vector.norm() == 0.0 {
            panic!("At least one of the entered vectors is zero!");
        }
        let orth_vector: Vector3<f64> = Vector3::cross(&beam_direction, &x_vector).normalize();
        let x_vector = x_vector.normalize();
        Frame {
            x_vector,
            y_vector: orth_vector,
        }
    }
}
