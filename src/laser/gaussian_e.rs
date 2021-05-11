//! Gaussian beam intensity distribution

extern crate nalgebra;
extern crate num;
extern crate rayon;
extern crate specs;
use crate::atom::Position;
use crate::constant::PI;
use crate::laser::gaussian;
use crate::maths;
use nalgebra::Vector3;
use num::complex::Complex;
use serde::{Deserialize, Serialize};
use specs::{Component, HashMapStorage};

/// A component representing an electric field of a gaussian beam.
///
/// Interference effects will be possible to use with two or more instances of this component.
/// Infinite coherence is assumed, so far only (fully) linear polarization is respected - as it
/// is most common in optical dipole traps. Circular and elliptical polarization will be implemented via
/// extra components that rotate / scale the respective linear vector in the future
///
/// Other than the existing `GaussianBeam` component, the full wave-nature of light is respected internally here.
///
/// The beam will propagate in vacuum. Inhomogenous media, gravitational lensing, refractions and
/// reflections are not implemented.
///
/// Also, attenuation effects are not yet implemented but they might come in a version
/// that accounts for atom-atom intereactions in the future.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct LinearGaussianEBeam {
    /// A point that the laser beam intersects
    pub intersection: Vector3<f64>,

    /// Direction the beam propagates with respect to cartesian `x,y,z` axes.
    pub direction: Vector3<f64>,

    /// Radius of the beam at which the intensity is 1/e of the peak value, SI units of m.
    ///
    /// Since in the literature the e^2_radius (where intensity is 1/e^2 of peak value) is used
    /// very often as well, it is useful to note the following relation:
    ///
    /// e_radius = e^2_radius / sqrt(2)
    pub e_radius: f64,

    /// Power of the laser in W
    pub power: f64,

    /// Intensity amplitude in W/m^2
    pub intensity_0: f64,

    /// Electric field vector amplitude in
    pub e_0: Vector3<f64>,

    /// wavelength of the light in m
    pub wavelength: f64,

    /// wavenumber in 1/m
    pub wavenumber: f64,

    /// The distance along the propagation direction of a beam from the
    ///  waist to the place where the area of the cross section is doubled in units of metres
    pub rayleigh_range: f64,
}

impl Component for LinearGaussianEBeam {
    type Storage = HashMapStorage<Self>;
}

impl LinearGaussianEBeam {
    /// Create a GaussianBeam component by specifying the peak intensity, rather than power.
    ///
    /// # Arguments:
    ///
    /// `intersection`: as per component.
    ///
    /// `direction`: as per component.
    ///
    /// `polarization_direction`: initial direction into which the E-field is polarized.
    ///
    /// `power`: power of the beam in W
    ///
    /// `e_radius`: radius of beam in units of m.
    ///
    /// `wavelength`: wavelength of the electromagnetic light
    pub fn from_power(
        intersection: Vector3<f64>,
        direction: Vector3<f64>,
        polarization_direction: Vector3<f64>,
        power: f64,
        e_radius: f64,
        wavelength: f64,
    ) -> Self {
        let intensity = power / (PI * e_radius.powf(2.0));
        let e_0 = (2.0 * 377.0 * intensity).powf(0.5) * polarization_direction.normalize();
        LinearGaussianEBeam {
            intersection: intersection,
            direction: direction.normalize(),
            e_radius: e_radius,
            power: power,
            intensity_0: intensity,
            e_0: e_0,
            wavelength: wavelength,
            wavenumber: 2.0 * PI / wavelength,
            rayleigh_range: gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        }
    }
}

/// Returns the intensity of a gaussian laser beam at the specified position.
pub fn get_gaussian_e_field(beam: &LinearGaussianEBeam, pos: &Position) -> Complex<Vector3<f64>> {
    let (r, z) =
        maths::get_minimum_distance_line_point(&pos.pos, &beam.intersection, &beam.direction);

    let spot_size =
        2.0_f64.powf(0.5) * beam.e_radius * (1.0 + (z / beam.rayleigh_range).powf(2.0)).powf(0.5);
    let curvature = z + beam.rayleigh_range.powf(2.0) / z;
    let gouy_phase = (z / beam.rayleigh_range).atan();

    let float_amplitude: f64 =
        beam.e_radius * 2.0_f64.powf(0.5) / spot_size * (-(r / spot_size).powf(2.0)).exp();
    let real_amplitude: Complex<f64> = Complex::new(float_amplitude, 0.0);

    let phase_factor = Complex::new(
        0.0,
        beam.wavenumber * z + beam.wavenumber * r.powf(2.0) / (2.0 * curvature) - gouy_phase,
    )
    .exp();

    Complex::new(
        beam.e_0 * real_amplitude.re * phase_factor.re,
        beam.e_0 * real_amplitude.re * phase_factor.im,
    )
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;

    extern crate nalgebra;
    use nalgebra::Vector3;

    #[test]
    fn test_get_gaussian_e_field() {
        let beam = LinearGaussianEBeam::from_power(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
            1.0,
            100.0e-6 / 2.0_f64.powf(0.5),
            1064.0e-9,
        );
        let pos1 = Position {
            pos: Vector3::new(10.0e-6, 20.0e-6, 30.0e-6),
        };

        let e_field: Complex<Vector3<f64>> = get_gaussian_e_field(&beam, &pos1);
        assert_approx_eq!(e_field.re[0], 70182.09940488, 1e-6_f64);
        assert_approx_eq!(e_field.re[1], 0.0, 1e-6_f64);
        assert_approx_eq!(e_field.re[2], 0.0, 1e-6_f64);
        assert_approx_eq!(e_field.im[0], 196233.6664737, 1e-6_f64);
        assert_approx_eq!(e_field.im[1], 0.0, 1e-6_f64);
        assert_approx_eq!(e_field.im[2], 0.0, 1e-6_f64);
    }
}
