//! Magnetic field from a circular coil

extern crate nalgebra;
extern crate specs;

use std::f64::consts::PI;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use crate::atom::Position;
use crate::magnetic::MagneticFieldSampler;
use crate::maths::{ellip_e_approx, ellip_k_approx};

/// A component representing a coil.
#[derive(Serialize, Deserialize)]
pub struct MagneticCoil {
    /// Radius of the coil, in m.
    pub radius: f64,
    /// Current in the coil, in Ampere.
    pub current: f64,
    /// A unitary vector orthogonal to the coil surface.
    /// The current is positive if it is right-hand oriented with respect to the normal.
    pub normal: Vector3<f64>,
}

impl Component for MagneticCoil {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include coils in the world.
pub struct SampleCoilFieldSystem;

impl SampleCoilFieldSystem {
    /// Calculates the magnetic field of the coil.
    ///
    /// # Arguments
    ///
    /// `location`: position of the sampler, m
    ///
    /// `position`: position of the coil, m
    ///
    /// `radius`: radius of the coil, m
    ///
    /// `current`: current in the coil, Ampere
    ///
    /// `normal`: _normalised_ vector normal to the coil
    pub fn calculate_field(
        location: Vector3<f64>,
        position: Vector3<f64>,
        radius: f64,
        current: f64,
        normal: Vector3<f64>,
    ) -> Vector3<f64> {
        let delta = location - position;
        let b0 = 4.0 * PI * 1e-7 * current / 2.0 / radius;

        let ex = normal.clone();
        let x = normal.dot(&delta);

        let perp = delta - delta.dot(&ex) * ex;
        let r: f64 = perp.norm();
        let er = perp / (r + 1e-6);

        let alpha = r / radius;
        let beta = x / radius;
        let gamma = x / (r + 1e-6 * radius);
        let q = (1.0 + alpha).powi(2) + beta.powi(2);
        let k = (4.0 * alpha / q).sqrt();

        let ek = ellip_e_approx(k);
        let kk = ellip_k_approx(k);

        let bx = b0 / PI / q.sqrt() * (ek * (1.0 - alpha.powi(2) - beta.powi(2)) / (q - 4.0 * alpha)
            + kk);
        let br = b0 / PI / q.sqrt() * gamma * (ek * (1.0 + alpha.powi(2) + beta.powi(2)) / (q - 4.0 * alpha)
            - kk);

        return bx * normal + br * er
    }
}

impl<'a> System<'a> for SampleCoilFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, MagneticCoil>,
    );
    fn run(&mut self, (mut sampler, positions, coils): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (position, coil) in (&positions, &coils).join() {
            (&positions, &mut sampler)
                .par_join()
                .for_each(|(location, mut sampler)| {
                    let field = SampleCoilFieldSystem::calculate_field(
                        location.pos,
                        position.pos,
                        coil.radius,
                        coil.current,
                        coil.normal,
                    );
                    sampler.field = sampler.field + field;
                });
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    extern crate nalgebra;
    use nalgebra::Vector3;
    use assert_approx_eq::assert_approx_eq;

    /// Tests the correct implementation of the coil field
    #[test]
    fn test_coil_field() {
        let pos = Vector3::new(1.0, 0.0, 1.0);
        let centre = Vector3::new(0., 0., 0.);
        let radius = 2.0 * PI;
        let current = 3e7;
        let normal = Vector3::z();
        let field =
            SampleCoilFieldSystem::calculate_field(pos, centre, radius, current, normal);
        // Compare to true field value, but when using the approximation for
        // the elliptic integrals it is only valid to about 1e-4
        assert_approx_eq!(field.x, 0.1119068, 1e-4);
        assert_approx_eq!(field.y, 0.0);
        assert_approx_eq!(field.z, 2.9372828, 1e-4);
    }
}
