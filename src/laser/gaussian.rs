//! Gaussian beam intensity distribution

extern crate nalgebra;
extern crate rayon;
extern crate specs;
use crate::laser::frame::Frame;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage};

use crate::atom::Position;
use crate::constant::EXP;
use crate::constant::PI;
use crate::maths;
use crate::ramp::Lerp;
use serde::{Deserialize, Serialize};

/// A component representing an intensity distribution with a gaussian profile.
///
/// The beam will propagate in vacuum. Inhomogenous media, gravitational lensing, refractions and
/// reflections (other than through a `CircularMask` are not implemented.
///
/// Also, attenuation effects are not yet implemented but they might come in a version
/// that accounts for atom-atom intereactions in the future.
#[derive(Deserialize, Serialize, Clone, Copy, Lerp)]
pub struct GaussianBeam {
    /// A point that the laser beam intersects
    pub intersection: Vector3<f64>,

    /// Direction the beam propagates, the axis direction of the gaussian beam. 
    /// the transverse direction is not defined here, they will be defined by frame vectors when extracting 
    /// gaussian intensity and gaussian intensity gradient
    pub direction: Vector3<f64>,
    
    /// Power of the laser in W
    pub power: f64,

    /// standard gaussian beam waist (1/e^2 radius) in units of m.
    pub w0_x: f64,
    pub w0_y: f64,

    /// Rayleigh range of the elliptical beam in units of m.
    pub rayleigh_range_x: f64,
    pub rayleigh_range_y: f64,

}
impl Component for GaussianBeam {
    type Storage = HashMapStorage<Self>;
}
impl GaussianBeam {
    /// Create a GaussianBeam component by specifying the peak intensity, rather than power.
    ///
    /// # Arguments:
    ///
    /// `intersection`: as per component.
    ///
    /// `direction`: as per component.
    ///
    /// `peak_intensity`: peak intensity in units of W/m^2.
    ///
    /// `w0_x, w0_y`: radius of beam in units of m.

    /// Here we are defining and unphysical beam, kinda "collimated" beam,
    /// this bit is used in laser cooling modules.
    pub fn from_peak_intensity(
        intersection: Vector3<f64>,
        direction: Vector3<f64>,
        peak_intensity: f64,
        w0_x: f64,
        w0_y: f64,
    ) -> Self {
        let power = 1.0/2.0 * std::f64::consts::PI * w0_x * w0_y * peak_intensity;
        GaussianBeam {
            intersection,
            direction,
            power,
            w0_x,
            w0_y,
            rayleigh_range_x: f64::INFINITY,
            rayleigh_range_y: f64::INFINITY,

        }
    }
}

impl GaussianBeam {
    /// Create a GaussianBeam component by specifying the peak intensity, rather than power.
    ///
    /// # Arguments:
    ///
    /// `intersection`: as per component.
    ///
    /// `direction`: as per component.
    ///
    /// `peak_intensity`: peak intensity in units of W/m^2.
    ///
    /// `w0_x, w0_y`: radius of beam in units of m.
    ///
    /// `wavelength`: wavelength of the electromagnetic light
    pub fn from_peak_intensity_ellipticity_with_rayleigh_range(
        intersection: Vector3<f64>,
        direction: Vector3<f64>,
        peak_intensity: f64,
        w0_x: f64,
        w0_y: f64,
        wavelength: f64,
    ) -> Self {
        let power = std::f64::consts::PI * w0_x * w0_y * peak_intensity / 2.0;
        GaussianBeam {
            intersection,
            direction,
            power,
            w0_x,
            w0_y,
            rayleigh_range_x: calculate_rayleigh_range(&wavelength, &w0_x),
            rayleigh_range_y: calculate_rayleigh_range(&wavelength, &w0_y),
        }
    }
    /// Create a GaussianBeam component by specifying the peak intensity, rather than power.
    ///
    /// # Arguments:
    ///
    /// `intersection`: as per component.
    ///
    /// `direction`: as per component.
    ///
    /// `power`: power of the beam in W
    ///
    /// `e_radius`: radius of beam in units of m.
    ///
    /// `wavelength`: wavelength of the electromagnetic light
    ///
    /// `ellipticity`: sqrt(1-(b/a)^2) measures the ellipticity of the intensity profile. Is zero for symmetric beams.
    pub fn from_power_with_ellipticity_and_rayleigh_range(
        intersection: Vector3<f64>,
        direction: Vector3<f64>,
        power: f64,
        w0_x: f64,
        w0_y: f64,
        wavelength: f64,
    ) -> Self {
        GaussianBeam {
            intersection,
            direction: direction.normalize(),
            power,
            w0_x,
            w0_y,
            rayleigh_range_x: calculate_rayleigh_range(&wavelength, &w0_x),
            rayleigh_range_y: calculate_rayleigh_range(&wavelength, &w0_y),
        }
    }
}

/// A component that covers the central portion of a laser beam.
///
/// The mask is assumed to be coaxial to the GaussianBeam.
#[derive(Clone, Copy)]
pub struct CircularMask {
    /// Radius of the masked region in units of m.
    pub radius: f64,
}
impl Component for CircularMask {
    type Storage = HashMapStorage<Self>;
}

/// Returns the intensity of a gaussian laser beam at the specified position.

pub fn get_gaussian_beam_intensity(
    beam: &GaussianBeam,
    pos: &Position,
    _mask: Option<&CircularMask>,
    frame: Option<&Frame>,
) -> f64 {
    /// Frame vector specify transverses direction of the beam, then convert pos (which is cartesian standard)
    /// into beam frame, then fed into standard gaussian intensity formula to extract intensity.
    let binding = Frame{
    x_vector: Vector3::x(),
    y_vector: Vector3::y(),
    };
    let frame = frame.unwrap_or(&binding);

    let (x, y, z) = maths::get_relative_coordinates_line_point(
        &pos.pos,
        &beam.intersection,
        &beam.direction,
        &frame,
    );

    2.0 * beam.power / PI / beam.w0_x / beam.w0_y / (1.0 + (z / beam.rayleigh_range_x).powf(2.0)).powf(0.5) 
        / (1.0 + (z / beam.rayleigh_range_y).powf(2.0)).powf(0.5)
        * EXP.powf(
            -2.0 * x.powf(2.0) / (beam.w0_x.powf(2.0) * (1. + (z / beam.rayleigh_range_x).powf(2.0)))
            -2.0 * y.powf(2.0) / (beam.w0_y.powf(2.0) * (1. + (z / beam.rayleigh_range_y).powf(2.0)))
        )
}
/// Computes the rayleigh range for a given beam and wavelength, w0 is standard 1/e^2 gaussian beam waist.
pub fn calculate_rayleigh_range(wavelength: &f64, w0: &f64) -> f64 {
    PI * w0.powf(2.0) / wavelength
}


/// Returns the intensity of a gaussian laser beam at the specified position.
// pub fn get_gaussian_beam_intensity(
//     beam: &GaussianBeam,
//     pos: &Position,
//     mask: Option<&CircularMask>,
//     frame: Option<&Frame>,
// ) -> f64 {
//     let (z, distance_squared) = match frame {
//         // checking if frame is given (for calculating ellipticity)
//         Some(frame) => {
//             let (x, y, z) = maths::get_relative_coordinates_line_point(
//                 &pos.pos,
//                 &beam.intersection,
//                 &beam.direction,
//                 frame,
//             );
//             let semi_major_axis = 1.0 / (1.0 - beam.ellipticity.powf(2.0)).powf(0.5);

//             // the factor (1.0 / semi_major_axis) is necessary so the overall power of the beam is not changed.
//             (
//                 z,
//                 (1.0 / semi_major_axis) * ((x).powf(2.0) + (y * semi_major_axis).powf(2.0)),
//             )
//         }
//         // ellipticity will be ignored (i.e. treated as zero) if no `Frame` is supplied.
//         None => {
//             let (distance, z) = maths::get_minimum_distance_line_point(
//                 &pos.pos,
//                 &beam.intersection,
//                 &beam.direction,
//             );
//             (z, distance * distance)
//         }
//     };
//     let power = match mask {
//         Some(mask) => {
//             if distance_squared.powf(0.5) < mask.radius {
//                 0.0
//             } else {
//                 beam.power
//             }
//         }
//         None => beam.power,
//     };
//     power / PI / beam.e_radius.powf(2.0) / (1.0 + (z / beam.rayleigh_range).powf(2.0))
//         * EXP.powf(
//             -distance_squared
//                 / (beam.e_radius.powf(2.0) * (1. + (z / beam.rayleigh_range).powf(2.0))),
//         )
// }


/// Computes the intensity gradient of a given beam and returns it as
/// a three-dimensional vector
pub fn get_gaussian_beam_intensity_gradient(
    beam: &GaussianBeam,
    pos: &Position,
    reference_frame: &Frame,
) -> Vector3<f64> {
    // Relative coordinate from beam intersection
    let rela_coord = pos.pos - beam.intersection;

    // Ellipticity treatment: waists along long and short axes
    let wx = beam.w0_x;      // long axis waist squared
    let wy = beam.w0_y;              // short axis waist squared

    // Rayleigh ranges for each axis (waist squared)
    let zx = beam.rayleigh_range_x;
    let zy = beam.rayleigh_range_y; // convert for long axis

    // Local coordinates along beam axes
    let x = rela_coord.dot(&reference_frame.x_vector);
    let y = rela_coord.dot(&reference_frame.y_vector);
    let z = rela_coord.dot(&beam.direction);

    // z-dependent spot sizes
    let wx_z = wx * ( (1.0 + (z / zx).powi(2))).sqrt();
    let wy_z = wy * ( (1.0 + (z / zy).powi(2))).sqrt();

    // Intensity prefactor
    let intensity_prefactor = 2.0 * beam.power / std::f64::consts::PI / (wx_z * wy_z);

    // Exponential factor
    let exp_factor = (-2.0 * (x.powi(2) / wx_z.powi(2) + y.powi(2) / wy_z.powi(2))).exp();

    // Gradient along x and y
    let grad_x = -4.0 * x / wx_z.powi(2) * intensity_prefactor * exp_factor;
    let grad_y = -4.0 * y / wy_z.powi(2) * intensity_prefactor * exp_factor;

    // Gradient along z
    // Term 1: derivative of prefactor 1/(wx_z wy_z)
    let term1 = -intensity_prefactor / (wx_z * wy_z) 
    * (
        z * wx.powi(2) * wy_z/ (zx.powi(2) * wx_z) +
        z * wy.powi(2) * wx_z / (zy.powi(2) * wy_z)
    );

    // Term 2: derivative of exponential
    let term2 = intensity_prefactor * 4.0 * (
        x.powi(2) * z * wx.powi(2) / (wx_z.powi(4) * zx.powi(2)) +
        y.powi(2) * z * wy.powi(2) / (wy_z.powi(4) * zy.powi(2))
    );

    let grad_z = (term1 + term2) * exp_factor;

    // Combine into Vector3
    reference_frame.x_vector * grad_x
        + reference_frame.y_vector * grad_y
        + beam.direction * grad_z
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::constant::PI;
    use assert_approx_eq::assert_approx_eq;

    extern crate nalgebra;
    use nalgebra::Vector3;

    #[test]
    fn test_get_gaussian_beam_intensity_gradient() {
        let beam = GaussianBeam {
            direction: Vector3::z(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            w0_x: 70.71067812e-6,
            w0_y: 70.71067812e-6,
            power: 100.0,
            rayleigh_range_x: calculate_rayleigh_range(&1064.0e-9, &70.71067812e-6),
            rayleigh_range_y: calculate_rayleigh_range(&1064.0e-9, &70.71067812e-6),
        };
        let pos1 = Position {
            pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
        };
        let grf = Frame {
            x_vector: Vector3::x(),
            y_vector: Vector3::y(),
        };

        let gradient = get_gaussian_beam_intensity_gradient(&beam, &pos1, &grf);

        println!("Gradient: {:?}", gradient);

        assert_approx_eq!(gradient[0], -97864416562438.33, 1e+8_f64);
        assert_approx_eq!(gradient[1], 0.0, 1e+9_f64);
        assert_approx_eq!(gradient[2], -3232964116.6507816, 1e+6_f64);
        
    }

    #[test]
    fn test_get_gaussian_beam_intensity() {
        let beam = GaussianBeam {
            direction: Vector3::x(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            w0_x: 2.0,
            w0_y: 2.0,
            power: 1.0,
            rayleigh_range_x: calculate_rayleigh_range(&1064.0e-9, &2.0),
            rayleigh_range_y: calculate_rayleigh_range(&1064.0e-9, &2.0),
        };

        let pos1 = Position { pos: Vector3::x() };
        assert_approx_eq!(
            beam.power
                / (PI.powf(0.5) * beam.w0_x.sqrt() * PI.powf(0.5) * beam.w0_y.sqrt())
                / (1.0 + 1.0 / calculate_rayleigh_range(&1064.0e-9, &2.0).powf(2.0)),
            get_gaussian_beam_intensity(&beam, &pos1, None, None),
            1e-6_f64
        );

        let pos2 = Position { pos: Vector3::y() };
        assert_approx_eq!(
            1.0 / (PI.powf(0.5) * beam.w0_x.sqrt() * PI.powf(0.5) * beam.w0_y.sqrt())
                * (-pos2.pos[1] / beam.w0_x.sqrt() / beam.w0_y.sqrt()).exp(),
            get_gaussian_beam_intensity(&beam, &pos2, None, None),
            1e-6_f64
        );

        assert_approx_eq!(
            beam.power
                / (PI.powf(0.5) * beam.w0_x/(2.0f64).sqrt() * PI.powf(0.5) * beam.w0_y/(2.0f64).sqrt())
                / (1.0 + 1.0 / calculate_rayleigh_range(&1064.0e-9, &(2.0*(2.0f64).sqrt())).powf(2.0)),
            get_gaussian_beam_intensity(&beam, &pos1, None, None),
            1e-6_f64
        );

        assert_approx_eq!(
            1.0 / (PI.powf(0.5) * beam.w0_x.sqrt() * PI.powf(0.5) * beam.w0_y.sqrt())
                * (-pos2.pos[1] / beam.w0_x.sqrt() / beam.w0_y.sqrt()).exp(),
            get_gaussian_beam_intensity(&beam, &pos2, None, None),
            1e-6_f64
        );


        let beam2 = GaussianBeam {
            direction: Vector3::z(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            w0_x: 2.0,
            w0_y: 2.0,
            power: 1.0,
            rayleigh_range_x: calculate_rayleigh_range(&1064.0e-9, &2.0),
            rayleigh_range_y: calculate_rayleigh_range(&1064.0e-9, &2.0),
        };

        let pos4 = Position { pos: Vector3::new(1.0, 2.0, 3.0) };
        let intensity_from_rust = get_gaussian_beam_intensity(&beam2, &pos4, None, None);
        println!("Intensity from rust: {}", intensity_from_rust);
        assert_approx_eq!(
            0.013064233284686179,
            intensity_from_rust,
            1e-12_f64
        );

    }
}
