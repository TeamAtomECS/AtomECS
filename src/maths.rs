//! Mathematical utilities

extern crate nalgebra;
extern crate rand;

use nalgebra::Vector3;
use rand::Rng;

use crate::constant::EXP;
use crate::constant::PI;

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
) -> f64 {
    let rela_cood = pos - line_point;
    let distance = (dir.cross(&rela_cood) / dir.norm()).norm();
    distance
}

/// A normalised gaussian distribution.
///
/// The distribution is normalised such that the 2D area underneath a gaussian dist with sigma_x=sigma_y=std is equal to 1.
pub fn gaussian_dis(std: f64, distance: f64) -> f64 {
    1.0 / (2.0 * PI * std * std) * EXP.powf(-distance * distance / 2.0 / (std * std))
}

/// generate a uniform random direction
pub fn random_direction() -> Vector3<f64> {
    let mut rng = rand::thread_rng();
    let angle1 = rng.gen_range(0.0, PI);
    let angle2 = rng.gen_range(0., 2. * PI);
    let result = Vector3::new(
        angle1.cos(),
        angle1.sin() * angle2.sin(),
        angle1.sin() * angle2.cos(),
    );
    //println!("{:?}",result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_distance_line_point() {
        let pos = Vector3::new(1., 1., 1.);
        let centre = Vector3::new(0., 1., 1.);
        let dir = Vector3::new(1., 2., 2.);
        let distance = get_minimum_distance_line_point(&pos, &centre, &dir);
        assert!(distance > 0.942, distance < 0.943);
    }
}
