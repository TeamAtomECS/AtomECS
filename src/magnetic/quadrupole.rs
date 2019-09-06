
extern crate nalgebra;
extern crate specs;
use crate::atom::Position;

use crate::magnetic::MagneticFieldSampler;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};
/// A component representing a 3D quadrupole field.
pub struct QuadrupoleField3D {
    /// Gradient of the quadrupole field, in units of Tesla/m
    pub gradient: f64,
    pub direction: Vector3<f64>,
}
impl QuadrupoleField3D {
    /// Creates a `QuadrupoleField3D` component with gradient specified in Gauss per cm.
    pub fn gauss_per_cm(gradient: f64,direction:&Vector3<f64>) -> Self {
        Self {
            gradient: gradient * 0.01,
            direction:direction.clone()
        }
    }
}

impl Component for QuadrupoleField3D {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include quadrupole fields in the world.
pub struct Sample3DQuadrupoleFieldSystem;

impl Sample3DQuadrupoleFieldSystem {
    /// Calculates the quadrupole magnetic field.
    /// The field is defined with components `Bx = grad*x`, `By = grad*y`, `Bz = -2 * grad * z`.
    ///
    /// # Arguments
    ///
    /// `pos`: position of the sampler, m
    ///
    /// `centre`: position of the quadrupole node, m
    ///
    /// `gradient`: quadrupole gradient, in Tesla/m
    pub fn calculate_field(
        pos: &Vector3<f64>,
        centre: &Vector3<f64>,
        gradient: f64,
        direction:&Vector3<f64>
    ) -> Vector3<f64> {
        let rel_pos = pos - centre;
        let dir = direction.normalize();
        let dis_para = rel_pos.dot(&dir);
        let new_dir = Vector3::new(1.212,2.31,0.4123).normalize();
        let dir_1 = dir.cross(&new_dir);
        let dir_2 = dir.cross(&dir_1);
        let dis_per_1 = rel_pos.dot(&dir_1);
        let dis_per_2 = rel_pos.dot(&dir_2);
        gradient*(dir_1 * dis_per_1 + dir_2*dis_per_2 -2.0 * dir * dis_para)
    }
}

impl<'a> System<'a> for Sample3DQuadrupoleFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, QuadrupoleField3D>,
    );
    fn run(&mut self, (mut sampler, pos, quadrupole): Self::SystemData) {
        for (centre, quadrupole) in (&pos, &quadrupole).join() {
            for (pos, mut sampler) in (&pos, &mut sampler).join() {
                let quad_field = Sample3DQuadrupoleFieldSystem::calculate_field(
                    &pos.pos,
                    &centre.pos,
                    quadrupole.gradient,
                    &quadrupole.direction
                );
                sampler.field = sampler.field + quad_field;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Tests the correct implementation of the quadrupole 3D field
    #[test]
    fn test_quadrupole3dfield() {
        let pos = Vector3::new(1.0, 1.0, 1.0);
        let centre = Vector3::new(0., 1., 0.);
        let gradient = 1.;
        let field = Sample3DQuadrupoleFieldSystem::calculate_field(&pos, &centre, gradient,&Vector3::new(0.,0.,1.0));
        assert_eq!(field, Vector3::new(1., 0., -2.));
    }
}
