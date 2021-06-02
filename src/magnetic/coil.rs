//! Magnetic field from a circular coil

extern crate nalgebra;

use std::f64::consts::PI;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use crate::atom::Position;
use crate::magnetic::MagneticFieldSampler;

/// A component representing a circular coil made of a single loop.
#[derive(Serialize, Deserialize)]
pub struct MagneticCoilField {
    /// Radius of the coil, in m.
    pub radius: f64,
    /// Current in the coil, in Ampere.
    pub current: f64,
    /// A unitary vector orthogonal to the coil surface.
    /// The current is positive if it is right-hand oriented with respect to the normal.
    pub normal: Vector3<f64>,
}

impl Component for MagneticCoilField {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include coils in the world.
pub struct SampleMagneticCoilFieldSystem;

impl SampleMagneticCoilFieldSystem {
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

        let ellips = ellip_ke(k, 1e-6);
        let kk = ellips.0;
        let ek = ellips.1;

        let bx = b0 / PI / q.sqrt() * (ek * (1.0 - alpha.powi(2) - beta.powi(2)) / (q - 4.0 * alpha)
            + kk);
        let br = b0 / PI / q.sqrt() * gamma * (ek * (1.0 + alpha.powi(2) + beta.powi(2)) / (q - 4.0 * alpha)
            - kk);

        return bx * normal + br * er;
    }
}

impl<'a> System<'a> for SampleMagneticCoilFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, MagneticCoilField>,
    );
    fn run(&mut self, (mut sampler, positions, coils): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (position, coil) in (&positions, &coils).join() {
            (&positions, &mut sampler)
                .par_join()
                .for_each(|(location, mut sampler)| {
                    let field = SampleMagneticCoilFieldSystem::calculate_field(
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


/// Computes the complete elliptic integrals of first and second kind.
///
/// Conventions for the argument are the same as in
/// Carlson, B. C. (1995). "Numerical Computation of Real or Complex Elliptic Integrals". Numerical Algorithms. 10 (1): 13–26.
///
/// # Arguments
///
/// `k`: argument of the elliptic integrals, must be 0 <= k < 1.
///
/// `epsrel`: relative tolerable error for the function evaluation
fn ellip_ke(k: f64, epsrel: f64) -> (f64, f64)
{
    let mut a = 1.;
    let mut g = (1. - k.powi(2)).sqrt();
    let mut c = k;
    let mut power2_acc = 0.5;
    let mut c_acc = power2_acc * c.powi(2);
    loop {
        let a_new = (a + g) / 2.;
        let g_new = (a * g).sqrt();
        let c_new = c.powi(2) / 4. / a_new;
        power2_acc *= 2.;
        c_acc += power2_acc * c_new.powi(2);
        let agm_converged = (a_new - a).abs() <= epsrel.sqrt() * a_new;

        a = a_new;
        g = g_new;
        c = c_new;
        if agm_converged {
            break;
        }
    }
    let ellip_k = PI / 2. / a;
    let ellip_e = ellip_k * (1. - c_acc);
    return (ellip_k, ellip_e);
}

#[cfg(test)]
pub mod tests {
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;

    use super::*;

    extern crate nalgebra;

    /// Test computation of elliptic integrals
    #[test]
    fn test_elliptic() {
        let k = 0.5;
        let epsrel = 1e-6;
        let values = ellip_ke(k, epsrel);
        assert_approx_eq!(values.0, 1.685750354812596);
        assert_approx_eq!(values.1, 1.467462209339427);
    }

    /// Tests the correct implementation of the coil field
    #[test]
    fn test_coil_field() {
        let pos = Vector3::new(1.0, 0.0, 1.0);
        let centre = Vector3::new(0., 0., 0.);
        let radius = 2.0 * PI;
        let current = 3e7;
        let normal = Vector3::z();
        let field =
            SampleMagneticCoilFieldSystem::calculate_field(pos, centre, radius, current, normal);
        assert_approx_eq!(field.x, 0.1119068);
        assert_approx_eq!(field.y, 0.0);
        assert_approx_eq!(field.z, 2.9372828);
    }
}
