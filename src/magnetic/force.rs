//! Magnetic dipole force applied to an atom in an external magnetic field
//! Applies a force based on gradient of energy from the linear Zeeman effect to any
//! entity with a MagneticDipole component.
//! The magnetic force is not added by default to the builder so must be explicitly included and
//! must depend on the magnetics_gradient system.
#![allow(non_snake_case)]

use super::MagneticFieldSampler;
use crate::atom::Force;
use crate::constant;
use crate::integrator::AtomECSBatchStrategy;
use bevy::prelude::*;

/// Component that represents the magnetic dipole moment of an atom.
#[derive(Clone, Component)]
pub struct MagneticDipole {
    /// Product of Zeeman state mF & lande g-factor
    pub mFgF: f64,
}

pub fn apply_magnetic_forces(
    mut query: Query<(&mut Force, &MagneticFieldSampler, &MagneticDipole)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(mut force, sampler, dipole)| {
            let dipole_force = -dipole.mFgF * constant::BOHRMAG * sampler.gradient;
            force.force += dipole_force;
        })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;
    use nalgebra::{Matrix3, Vector3};

    //Test correct force in an external magnetic gradient
    #[test]
    fn test_apply_magnetic_force_system() {
        let mut app = App::new();
        app.add_systems(Update, apply_magnetic_forces);
        app.insert_resource(AtomECSBatchStrategy::default());

        let atom1 = app
            .world_mut()
            .spawn(MagneticFieldSampler {
                field: Vector3::new(0.0, 0.0, 0.0),
                magnitude: 2.0,
                gradient: Vector3::new(1.0, -0.5, 2.0),
                jacobian: Matrix3::zeros(),
            })
            .insert(MagneticDipole { mFgF: 0.5 })
            .insert(Force::default())
            .id();

        app.update();
        let force = app
            .world()
            .get_entity(atom1)
            .expect("entity not found")
            .get::<Force>()
            .expect("Force not found")
            .force;

        let real_force = Vector3::new(
            -0.5 * constant::BOHRMAG,
            0.25 * constant::BOHRMAG,
            -1.0 * constant::BOHRMAG,
        );
        assert_approx_eq!(force[0], real_force[0], 1e-10_f64);
        assert_approx_eq!(force[1], real_force[1], 1e-10_f64);
        assert_approx_eq!(force[2], real_force[2], 1e-10_f64);
    }
}
