//! Mathematical utilities

use crate::constant::EXP;
use crate::constant::PI;
extern crate nalgebra;
use nalgebra::Vector3;

/// Get relative coordinates between a point and a line.
///
/// # Arguments
///
/// `pos`: position of the point
///
/// `line_point`: a point on the line
///
/// `dir`: vector pointing along the line.
pub fn get_relative_coordinates_line_point(
    pos: &Vector3<f64>,
    line_point: &Vector3<f64>,
    dir: &Vector3<f64>,
    frame: &crate::laser::frame::Frame,
) -> (f64, f64, f64) {
    let rela_cood = pos - line_point;
    let z = rela_cood.dot(dir) / dir.norm();
    let r_vec = rela_cood - z * dir;
    let x = r_vec.dot(&frame.x_vector);
    let y = r_vec.dot(&frame.y_vector);
    (x, y, z)
}

/// Get miniminum distance between a point and a line.
///
/// # Arguments
///
/// `pos`: position of the point
///
/// `line_point`: a point on the line
///
/// `dir`: vector pointing along the line.
pub fn get_minimum_distance_line_point(
    pos: &Vector3<f64>,
    line_point: &Vector3<f64>,
    dir: &Vector3<f64>,
) -> (f64, f64) {
    let rela_cood = pos - line_point;
    let distance = (dir.cross(&rela_cood) / dir.norm()).norm();
    let z = rela_cood.dot(dir) / dir.norm();
    (distance, z)
}

/// A normalised gaussian distribution.
///
/// The distribution is normalised such that the 2D area underneath a gaussian dist with sigma_x=sigma_y=std is equal to 1.
pub fn gaussian_dis(std: f64, distance_squared: f64) -> f64 {
    1.0 / (2.0 * PI * std * std) * EXP.powf(-distance_squared / 2.0 / (std * std))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_distance_line_point() {
        let pos = Vector3::new(1., 1., 1.);
        let centre = Vector3::new(0., 1., 1.);
        let dir = Vector3::new(1., 2., 2.);
        let (distance, _) = get_minimum_distance_line_point(&pos, &centre, &dir);
        assert!(distance > 0.942, "{}", distance < 0.943);
    }
}
