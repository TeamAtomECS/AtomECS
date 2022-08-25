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

    /// The distance along the propagation direction of a beam from the
    ///  waist to the place where the area of the cross section is doubled in units of metres
    pub rayleigh_range: f64,

    /// ellipticity
    pub ellipticity: f64,
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
    /// `e_radius`: radius of beam in units of m.
    pub fn from_peak_intensity(
        intersection: Vector3<f64>,
        direction: Vector3<f64>,
        peak_intensity: f64,
        e_radius: f64,
    ) -> Self {
        let std = e_radius / 2.0_f64.powf(0.5);
        let power = 2.0 * std::f64::consts::PI * std.powi(2) * peak_intensity;
        GaussianBeam {
            intersection,
            direction,
            power,
            e_radius,
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
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
    /// `e_radius`: radius of beam in units of m.
    ///
    /// `wavelength`: wavelength of the electromagnetic light
    pub fn from_peak_intensity_with_rayleigh_range(
        intersection: Vector3<f64>,
        direction: Vector3<f64>,
        peak_intensity: f64,
        e_radius: f64,
        wavelength: f64,
    ) -> Self {
        let std = e_radius / 2.0_f64.powf(0.5);
        let power = 2.0 * std::f64::consts::PI * std.powi(2) * peak_intensity;
        GaussianBeam {
            intersection,
            direction,
            power,
            e_radius,
            rayleigh_range: calculate_rayleigh_range(&wavelength, &e_radius),
            ellipticity: 0.0,
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
        e_radius: f64,
        wavelength: f64,
        ellipiticity: f64,
    ) -> Self {
        GaussianBeam {
            intersection,
            direction: direction.normalize(),
            power,
            e_radius,
            rayleigh_range: calculate_rayleigh_range(&wavelength, &e_radius),
            ellipticity: ellipiticity,
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
    mask: Option<&CircularMask>,
    frame: Option<&Frame>,
) -> f64 {
    let (z, distance_squared) = match frame {
        // checking if frame is given (for calculating ellipticity)
        Some(frame) => {
            let (x, y, z) = maths::get_relative_coordinates_line_point(
                &pos.pos,
                &beam.intersection,
                &beam.direction,
                frame,
            );
            let semi_major_axis = 1.0 / (1.0 - beam.ellipticity.powf(2.0)).powf(0.5);

            // the factor (1.0 / semi_major_axis) is necessary so the overall power of the beam is not changed.
            (
                z,
                (1.0 / semi_major_axis) * ((x).powf(2.0) + (y * semi_major_axis).powf(2.0)),
            )
        }
        // ellipticity will be ignored (i.e. treated as zero) if no `Frame` is supplied.
        None => {
            let (distance, z) = maths::get_minimum_distance_line_point(
                &pos.pos,
                &beam.intersection,
                &beam.direction,
            );
            (z, distance * distance)
        }
    };
    let power = match mask {
        Some(mask) => {
            if distance_squared.powf(0.5) < mask.radius {
                0.0
            } else {
                beam.power
            }
        }
        None => beam.power,
    };
    power / PI / beam.e_radius.powf(2.0) / (1.0 + (z / beam.rayleigh_range).powf(2.0))
        * EXP.powf(
            -distance_squared
                / (beam.e_radius.powf(2.0) * (1. + (z / beam.rayleigh_range).powf(2.0))),
        )
}
/// Computes the rayleigh range for a given beam and wavelength
pub fn calculate_rayleigh_range(wavelength: &f64, e_radius: &f64) -> f64 {
    2.0 * PI * e_radius.powf(2.0) / wavelength
}

/// Computes the intensity gradient of a given beam and returns it as
/// a three-dimensional vector
pub fn get_gaussian_beam_intensity_gradient(
    beam: &GaussianBeam,
    pos: &Position,
    reference_frame: &Frame,
) -> Vector3<f64> {
    let rela_coord = pos.pos - beam.intersection;

    // ellipticity treatment
    let semi_major_axis = 1.0 / (1.0 - beam.ellipticity.powf(2.0)).powf(0.5);

    let x = rela_coord.dot(&reference_frame.x_vector) / semi_major_axis.powf(0.5);
    let y = rela_coord.dot(&reference_frame.y_vector) * semi_major_axis.powf(0.5);
    let z = rela_coord.dot(&beam.direction);

    let spot_size_squared =
        2.0 * beam.e_radius.powf(2.0) * (1. + (z / beam.rayleigh_range).powf(2.0));
    let vector = -4. * (reference_frame.x_vector * x + reference_frame.y_vector * y)
        + beam.direction * z / (beam.rayleigh_range.powf(2.0) + z.powf(2.0))
            * (- 2.0 * spot_size_squared + 4. * (x.powf(2.0) + y.powf(2.0)));
    let intensity = 2. * beam.power / PI / spot_size_squared
        * EXP.powf(-2. * (x.powf(2.0) + y.powf(2.0)) / spot_size_squared);

    intensity / spot_size_squared * vector
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
            e_radius: 70.71067812e-6,
            power: 100.0,
            rayleigh_range: calculate_rayleigh_range(&1064.0e-9, &70.71067812e-6),
            ellipticity: 0.0,
        };
        let pos1 = Position {
            pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
        };
        let grf = Frame {
            x_vector: Vector3::x(),
            y_vector: Vector3::y(),
        };

        let gradient = get_gaussian_beam_intensity_gradient(&beam, &pos1, &grf);
        assert_approx_eq!(gradient[0], -2.49605032e+13, 1e+8_f64);
        assert_approx_eq!(gradient[1], 0.0, 1e+9_f64);
        assert_approx_eq!(gradient[2], -2.06143366e+08, 1e+6_f64);
    }

    #[test]
    fn test_get_gaussian_beam_intensity() {
        let beam = GaussianBeam {
            direction: Vector3::x(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 2.0,
            power: 1.0,
            rayleigh_range: calculate_rayleigh_range(&1064.0e-9, &2.0),
            ellipticity: 0.0,
        };

        let pos1 = Position { pos: Vector3::x() };
        assert_approx_eq!(
            beam.power
                / (PI.powf(0.5) * beam.e_radius).powf(2.0)
                / (1.0 + 1.0 / calculate_rayleigh_range(&1064.0e-9, &2.0).powf(2.0)),
            get_gaussian_beam_intensity(&beam, &pos1, None, None),
            1e-6_f64
        );

        let pos2 = Position { pos: Vector3::y() };
        assert_approx_eq!(
            1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
                * (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
            get_gaussian_beam_intensity(&beam, &pos2, None, None),
            1e-6_f64
        );

        assert_approx_eq!(
            beam.power
                / (PI.powf(0.5) * beam.e_radius).powf(2.0)
                / (1.0 + 1.0 / calculate_rayleigh_range(&1064.0e-9, &2.0).powf(2.0)),
            get_gaussian_beam_intensity(&beam, &pos1, None, None),
            1e-6_f64
        );

        assert_approx_eq!(
            1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
                * (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
            get_gaussian_beam_intensity(&beam, &pos2, None, None),
            1e-6_f64
        );
        let rayleigh_range_2 = calculate_rayleigh_range(&1064.0e-6, &beam.e_radius);

        let pos3 = Position {
            pos: Vector3::x() * rayleigh_range_2,
        };

        // Test with a frame but ellipticity = 0
        let frame = Frame::from_direction(beam.direction, Vector3::new(0.0, 1.0, 0.0));
        assert_approx_eq!(
            beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0),
            get_gaussian_beam_intensity(&beam, &pos3, None, Some(&frame)),
            1e-6_f64
        );
        // Position along the focused axis
        let pos4 = Position {
            pos: Vector3::x() + Vector3::y(),
        };
        // Now with an ellipticity, that implies a/b = 2
        let beam = GaussianBeam {
            direction: Vector3::x(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 2.0,
            power: 1.0,
            rayleigh_range: calculate_rayleigh_range(&1064.0e-9, &2.0),
            ellipticity: (3.0 / 4.0_f64).powf(0.5),
        };

        // checking if value on x-axis stays the same (as without ellipticity and frame)
        assert_approx_eq!(
            beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0),
            get_gaussian_beam_intensity(&beam, &pos3, None, Some(&frame)),
            1e-6_f64
        );

        // manual calculation to get beam intensity
        let intensity_0 = beam.power / (PI * beam.e_radius.powf(2.0));
        let broadening = 1.0 / (1.0 + (1.0 / beam.rayleigh_range).powf(2.0));
        // factor of 0.5 in exponent because of rescaling the axis by a/b = 2
        let intensity =
            intensity_0 * broadening * EXP.powf(-0.5 * broadening / beam.e_radius.powf(2.0));

        assert_approx_eq!(
            intensity,
            get_gaussian_beam_intensity(&beam, &pos4, None, Some(&frame)),
            1e-6_f64
        );
        // now the ration is  a/b = 4
        let beam = GaussianBeam {
            direction: Vector3::x(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 2.0,
            power: 1.0,
            rayleigh_range: calculate_rayleigh_range(&1064.0e-9, &2.0),
            ellipticity: (15.0 / 16.0_f64).powf(0.5),
        };

        // but we check along the de-focused axis (so intensity is lower than in symmetrical case)
        let intensity =
            intensity_0 * broadening * EXP.powf(-4.0 * broadening / beam.e_radius.powf(2.0));
        assert_approx_eq!(
            intensity,
            get_gaussian_beam_intensity(
                &beam,
                &Position {
                    pos: Vector3::x() + Vector3::z(),
                },
                None,
                Some(&frame)
            ),
            1e-6_f64
        );
    }
}
