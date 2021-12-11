//! Magnetic quadrupole fields

extern crate nalgebra;
extern crate serde;
extern crate specs;
use crate::atom::Position;
use serde::Serialize;

use crate::magnetic::MagneticFieldSampler;
use crate::ramp::Lerp;
use nalgebra::{Matrix3, Unit, Vector3};
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

/// A component representing a 3D quadrupole field.
#[derive(Serialize, Clone, Copy, Lerp)]
pub struct QuadrupoleField3D {
    /// Gradient of the quadrupole field, in units of Tesla/m
    pub gradient: f64,
    /// A unit vector pointing along the symmetry axis of the 3D quadrupole field.
    pub direction: Vector3<f64>,
}
impl QuadrupoleField3D {
    /// Creates a `QuadrupoleField3D` component with gradient specified in Gauss per cm.
    #[inline]
    pub fn gauss_per_cm(gradient: f64, direction: Vector3<f64>) -> Self {
        Self {
            gradient: gradient * 0.01,
            direction: direction.normalize(),
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
    ///
    /// `direction`: A _normalized_ vector pointing in the direction of the quadrupole's symmetry axis.
    pub fn calculate_field(
        pos: Vector3<f64>,
        centre: Vector3<f64>,
        gradient: f64,
        direction: Vector3<f64>,
    ) -> Vector3<f64> {
        let delta = pos - centre;
        let z_comp = delta.dot(&direction) * direction;
        let r_comp = delta - z_comp;
        gradient * (r_comp - 2.0 * z_comp)
    }
}

impl<'a> System<'a> for Sample3DQuadrupoleFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, QuadrupoleField3D>,
    );
    fn run(&mut self, (mut sampler, pos, quadrupole): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (centre, quadrupole) in (&pos, &quadrupole).join() {
            (&pos, &mut sampler)
                .par_join()
                .for_each(|(pos, sampler)| {
                    let quad_field = Sample3DQuadrupoleFieldSystem::calculate_field(
                        pos.pos,
                        centre.pos,
                        quadrupole.gradient,
                        quadrupole.direction,
                    );
                    sampler.field += quad_field;

                    // calculate local jacobian for magnetic field gradient
                    let mut jacobian = Matrix3::<f64>::zeros();
                    let delta = 1e-9; // Is there a better way to choose this number?
                                      // Strictly speaking to be accurate it depends on the length scale over which
                                      // the magnetic field changes
                    for i in 0..3 {
                        let mut pos_plus_dr = pos.pos;
                        let mut pos_minus_dr = pos.pos;
                        pos_plus_dr[i] += delta;
                        pos_minus_dr[i] -= delta;

                        let b_plus_dr = Sample3DQuadrupoleFieldSystem::calculate_field(
                            pos_plus_dr,
                            centre.pos,
                            quadrupole.gradient,
                            quadrupole.direction,
                        );
                        let b_minus_dr = Sample3DQuadrupoleFieldSystem::calculate_field(
                            pos_minus_dr,
                            centre.pos,
                            quadrupole.gradient,
                            quadrupole.direction,
                        );

                        let grad_plus = (b_plus_dr - quad_field) / delta;
                        let grad_minus = (quad_field - b_minus_dr) / delta;
                        let gradient = (grad_plus + grad_minus) / 2.0;

                        jacobian.set_column(i, &gradient);
                    }
                    sampler.jacobian += jacobian;
                });
        }
    }
}

/// A component representing a 2D quadrupole field.
///
/// The quadrupole field is of the form `B = B'(x, -y, 0)`.
/// The coordinate system is aligned such that:
///  * `e_x` is in the direction `direction_out`
///  * `e_y` is in the direction `direction_in`.
pub struct QuadrupoleField2D {
    /// Gradient of the quadrupole field, `B'`, in units of Tesla/m
    pub gradient: f64,

    /// A unit vector that defines the direction along which the field lines point away from the node. Perpendicular to `axis` and `direction_in`.
    pub direction_out: Vector3<f64>,

    /// A unit vector that defines the direction along which the field lines point in from the node. Perpendicular to both `direction_out` and `axis`.
    pub direction_in: Vector3<f64>,
}
impl QuadrupoleField2D {
    /// Creates a `QuadrupoleField2D` component with gradient specified in Gauss/cm.
    pub fn gauss_per_cm(
        gradient: f64,
        axis: Unit<Vector3<f64>>,
        out_direction: Unit<Vector3<f64>>,
    ) -> Self {
        Self {
            direction_out: (out_direction.into_inner()
                - axis.into_inner() * axis.dot(&out_direction))
            .normalize(),
            direction_in: axis.cross(out_direction.as_ref()),
            gradient: gradient * 0.01,
        }
    }
}
impl Component for QuadrupoleField2D {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include 2d quadrupole fields in the world.
pub struct Sample2DQuadrupoleFieldSystem;
impl Sample2DQuadrupoleFieldSystem {
    /// Calculates 2D quadrupole magnetic field.
    ///
    /// # Arguments
    ///
    /// `pos`: position of the sampler, m
    ///
    /// `quad_pos`: position of the quadrupole entity, m
    ///
    /// `gradient`: quadrupole gradient, in Tesla/m
    ///
    /// `direction_in`: A unit vector in the direction for the field lines point in to the node.
    ///
    /// `direction_out`: A unit vector in the direction for the field lines point away from the node.
    pub fn calculate_field(
        pos: Vector3<f64>,
        quad_pos: Vector3<f64>,
        gradient: f64,
        direction_in: Vector3<f64>,
        direction_out: Vector3<f64>,
    ) -> Vector3<f64> {
        let delta = pos - quad_pos;
        let in_comp = direction_in.dot(&delta) * direction_in;
        let out_comp = direction_out.dot(&delta) * direction_out;
        gradient * (out_comp - in_comp)
    }
}

impl<'a> System<'a> for Sample2DQuadrupoleFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, QuadrupoleField2D>,
    );
    fn run(&mut self, (mut sampler, pos, quadrupole): Self::SystemData) {
        for (centre, quadrupole) in (&pos, &quadrupole).join() {
            for (pos, sampler) in (&pos, &mut sampler).join() {
                let quad_field = Self::calculate_field(
                    pos.pos,
                    centre.pos,
                    quadrupole.gradient,
                    quadrupole.direction_in,
                    quadrupole.direction_out,
                );
                sampler.field += quad_field;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    extern crate nalgebra;
    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;
    use specs::prelude::*;

    /// Tests the correct implementation of the quadrupole 3D field
    #[test]
    fn test_quadrupole_3d_field() {
        let pos = Vector3::new(1.0, 1.0, 1.0);
        let centre = Vector3::new(0., 1., 0.);
        let gradient = 1.;
        let field =
            Sample3DQuadrupoleFieldSystem::calculate_field(pos, centre, gradient, Vector3::z());
        assert_eq!(field, Vector3::new(1., 0., -2.));
    }

    #[test]

    fn test_quadrupole_jacobian_calculation() {
        let mut test_world = World::new();

        test_world.register::<QuadrupoleField3D>();
        test_world.register::<Position>();
        test_world.register::<MagneticFieldSampler>();

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.02, 0.01, -0.05),
            })
            .with(MagneticFieldSampler::default())
            .build();

        test_world
            .create_entity()
            .with(QuadrupoleField3D {
                gradient: 1.0,
                direction: Vector3::new(0.0, 0.0, 1.0),
            })
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();

        test_world
            .create_entity()
            .with(QuadrupoleField3D {
                gradient: 2.0,
                direction: Vector3::new(1.0, 0.0, 1.0).normalize(),
            })
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();

        let mut system = Sample3DQuadrupoleFieldSystem;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<MagneticFieldSampler>();

        let test_jacobian = sampler_storage
            .get(atom1)
            .expect("entity not found")
            .jacobian;

        assert_approx_eq!(test_jacobian[(0, 0)], 0.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(1, 0)], 0.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(2, 0)], -3.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(0, 1)], 0.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(1, 1)], 3.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(2, 1)], 0.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(0, 2)], -3.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(1, 2)], 0.0, 1e-6_f64);
        assert_approx_eq!(test_jacobian[(2, 2)], -3.0, 1e-6_f64);
    }

    #[test]
    fn test_quadrupole_2d_field() {
        let pos = Vector3::new(1.0, 1.0, 1.0);
        let centre = Vector3::new(0., 0.5, 0.);
        let gradient = 1.;
        let field = Sample2DQuadrupoleFieldSystem::calculate_field(
            pos,
            centre,
            gradient,
            Vector3::x(),
            Vector3::y(),
        );
        assert_eq!(field, Vector3::new(-1., 0.5, 0.));
    }
}
