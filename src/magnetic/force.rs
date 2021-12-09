//! Magnetic dipole force applied to an atom in an external magnetic field
//! Applies a force based on gradient of energy from the linear Zeeman effect to any
//! entity with a MagneticDipole component.
//! The magnetic force is not added by default to the builder so must be explicitly included and
//! must depend on the magnetics_gradient system.
#![allow(non_snake_case)]

use super::MagneticFieldSampler;
use crate::atom::Force;
use crate::constant;
use specs::{Component, ReadStorage, System, VecStorage, WriteStorage};

/// Component that represents the magnetic dipole moment of an atom.
#[derive(Clone)]
pub struct MagneticDipole {
    /// Product of Zeeman state mF & lande g-factor
    pub mFgF: f64,
}

impl Component for MagneticDipole {
    type Storage = VecStorage<Self>;
}

pub struct ApplyMagneticForceSystem;
impl<'a> System<'a> for ApplyMagneticForceSystem {
    type SystemData = (
        WriteStorage<'a, Force>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, MagneticDipole>,
    );

    fn run(&mut self, (mut forces, samplers, dipoles): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut forces, &samplers, &dipoles)
            .par_join()
            .for_each(|(force, sampler, dipole)| {
                let dipole_force = -dipole.mFgF * constant::BOHRMAG * sampler.gradient;
                force.force += dipole_force;
            });
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    extern crate specs;

    use assert_approx_eq::assert_approx_eq;
    use specs::prelude::*;
    extern crate nalgebra;
    use nalgebra::{Matrix3, Vector3};

    //Test correct force in an external magnetic gradient
    #[test]
    fn test_apply_magnetic_force_system() {
        let mut test_world = World::new();
        test_world.register::<MagneticFieldSampler>();
        test_world.register::<MagneticDipole>();
        test_world.register::<Force>();
        let atom1 = test_world
            .create_entity()
            .with(MagneticFieldSampler {
                field: Vector3::new(0.0, 0.0, 0.0),
                magnitude: 2.0,
                gradient: Vector3::new(1.0, -0.5, 2.0),
                jacobian: Matrix3::zeros(),
            })
            .with(MagneticDipole { mFgF: 0.5 })
            .with(Force::new())
            .build();

        let mut system = ApplyMagneticForceSystem;
        system.run_now(&test_world);
        test_world.maintain();
        let force_storage = test_world.read_storage::<Force>();
        let force = force_storage.get(atom1).expect("entity not found").force;

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
