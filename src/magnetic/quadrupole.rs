//! Magnetic quadrupole fields

use super::analytic::AnalyticField;
use bevy::prelude::*;
use nalgebra::{Unit, Vector3};

/// A component representing a 3D quadrupole field.
#[derive(Clone, Copy, Component)]
#[component(storage = "SparseSet")]
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
impl AnalyticField for QuadrupoleField3D {
    /// Calculates the quadrupole magnetic field.
    /// The field is defined with components `Bx = grad*x`, `By = grad*y`, `Bz = -2 * grad * z`.
    fn get_field(&self, origin: Vector3<f64>, field_point: Vector3<f64>) -> Vector3<f64> {
        let delta = field_point - origin;
        let z_comp = delta.dot(&self.direction) * self.direction;
        let r_comp = delta - z_comp;
        self.gradient * (r_comp - 2.0 * z_comp)
    }

    fn calculate_jacobian(&self) -> bool {
        true
    }
}

/// A component representing a 2D quadrupole field.
///
/// The quadrupole field is of the form `B = B'(x, -y, 0)`.
/// The coordinate system is aligned such that:
///  * `e_x` is in the direction `direction_out`
///  * `e_y` is in the direction `direction_in`.
#[derive(Clone, Copy, Component)]
#[component(storage = "SparseSet")]
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
impl AnalyticField for QuadrupoleField2D {
    fn get_field(&self, origin: Vector3<f64>, field_point: Vector3<f64>) -> Vector3<f64> {
        let delta = field_point - origin;
        let in_comp = self.direction_in.dot(&delta) * self.direction_in;
        let out_comp = self.direction_out.dot(&delta) * self.direction_out;
        self.gradient * (out_comp - in_comp)
    }

    fn calculate_jacobian(&self) -> bool {
        true
    }
}

#[cfg(test)]
pub mod tests {

    use crate::integrator::AtomECSBatchStrategy;

    use super::*;
    extern crate nalgebra;
    use assert_approx_eq::assert_approx_eq;

    /// Tests the correct implementation of the quadrupole 3D field
    #[test]
    fn test_quadrupole_3d_field() {
        let pos = Vector3::new(1.0, 1.0, 1.0);
        let centre = Vector3::new(0., 1., 0.);
        let quad_field = QuadrupoleField3D {
            gradient: 1.0,
            direction: Vector3::z(),
        };
        let field = quad_field.get_field(centre, pos);
        assert_eq!(field, Vector3::new(1., 0., -2.));
    }

    #[test]

    fn test_3d_quadrupole_systems() {
        use crate::atom::Position;
        use crate::magnetic::analytic::calculate_field_contributions;
        use crate::magnetic::MagneticFieldSampler;

        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());
        app.add_systems(Update, calculate_field_contributions::<QuadrupoleField3D>);

        let atom1 = app
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(0.02, 0.01, -0.05),
            })
            .insert(MagneticFieldSampler::default())
            .id();

        app.world_mut()
            .spawn(QuadrupoleField3D {
                gradient: 1.0,
                direction: Vector3::new(0.0, 0.0, 1.0),
            })
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            });

        app.world_mut()
            .spawn(QuadrupoleField3D {
                gradient: 2.0,
                direction: Vector3::new(1.0, 0.0, 1.0).normalize(),
            })
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            });

        app.update();

        let test_jacobian = app
            .world()
            .entity(atom1)
            .get::<MagneticFieldSampler>()
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
        let quad_field = QuadrupoleField2D {
            gradient,
            direction_out: Vector3::y(),
            direction_in: Vector3::x(),
        };
        let field = quad_field.get_field(centre, pos);
        assert_eq!(field, Vector3::new(-1., 0.5, 0.));
    }
}
