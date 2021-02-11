//! Implements the force of gravity.

use crate::atom::{Force, Mass};
use crate::constant;
use nalgebra::Vector3;
use specs::{Read, ReadStorage, System, WriteStorage};

/// A resource that indicates that the simulation should apply the force of gravity.
pub struct ApplyGravityOption;

/// This system adds the gravitational force to all entities with [Mass](struct.Mass.html).
pub struct ApplyGravitationalForceSystem;
impl<'a> System<'a> for ApplyGravitationalForceSystem {
    type SystemData = (
        WriteStorage<'a, Force>,
        ReadStorage<'a, Mass>,
        Option<Read<'a, ApplyGravityOption>>,
    );

    fn run(&mut self, (mut force, mass, gravity_option): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        match gravity_option {
            None => (),
            Some(_) => {
                (&mut force, &mass)
                    .par_join()
                    .for_each(|(mut force, mass)| {
                        force.force = force.force
                            + mass.value * constant::AMU * constant::GC * Vector3::new(0., 0., -1.);
                    });
            }
        }
    }
}
