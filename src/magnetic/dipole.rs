//! Magnetic field from a dipole.

extern crate nalgebra;
extern crate specs;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use crate::atom::Position;
use crate::magnetic::MagneticFieldSampler;

/// A component representing a dipole.
/// For example, this can be used to reproduce the field generated by a permanent magnet.
#[derive(Serialize, Deserialize)]
pub struct MagneticDipole {
    /// Moment of the dipole, in units of Ampere * m ^ 2
    pub moment: f64,
    /// A unit vector pointing along the direction of the dipole.
    pub direction: Vector3<f64>,
}

impl Component for MagneticDipole {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include dipoles in the world.
pub struct SampleDipoleFieldSystem;

impl SampleDipoleFieldSystem {
    /// Calculates the magnetic field of the dipole.
    ///
    /// # Arguments
    ///
    /// `location`: position of the sampler, m
    ///
    /// `position`: position of the dipole, m
    ///
    /// `moment`: moment of the dipole, in Ampere * m ^ 2
    ///
    /// `direction`: A _normalized_ vector pointing in the direction of the dipole.
    pub fn calculate_field(
        location: Vector3<f64>,
        position: Vector3<f64>,
        moment: f64,
        direction: Vector3<f64>,
    ) -> Vector3<f64> {
        let delta = location - position;
        let distance = delta.norm();
        let dir = 3.0 * delta * delta.dot(&direction) / distance.powi(5) - direction / distance.powi(3);
        1e-7 * moment * dir
    }
}

impl<'a> System<'a> for SampleDipoleFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, MagneticDipole>,
    );
    fn run(&mut self, (mut sampler, positions, dipoles): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (position, dipole) in (&positions, &dipoles).join() {
            (&positions, &mut sampler)
                .par_join()
                .for_each(|(location, mut sampler)| {
                    let field = SampleDipoleFieldSystem::calculate_field(
                        location.pos,
                        position.pos,
                        dipole.moment,
                        dipole.direction.normalize(),
                    );
                    sampler.field = sampler.field + field;
                });
        }
    }
}

#[cfg(test)]
pub mod tests {
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;

    use super::*;

    extern crate nalgebra;

    /// Tests the correct implementation of the computed dipole field.
    #[test]
    fn test_dipole_field() {
        let location = Vector3::new(1.0 / 2f64.sqrt(), 0., 1.0 / 2f64.sqrt());
        let position = Vector3::new(0., 0., 0.);
        let moment = 1e7;
        let direction = Vector3::z();
        let field =
            SampleDipoleFieldSystem::calculate_field(location, position, moment, direction);
        assert_approx_eq!(field.x, 1.5);
        assert_approx_eq!(field.y, 0.0);
        assert_approx_eq!(field.z, 0.5);
    }
}