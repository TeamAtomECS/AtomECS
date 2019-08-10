use crate::atom::{Force, Mass};
use specs::{Join, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use crate::constant;
use nalgebra::Vector3;

/// Applies the force of gravity to all entities with mass.
pub struct ApplyGravitationalForceSystem;
impl <'a> System <'a> for ApplyGravitationalForceSystem{
    type SystemData = ( WriteStorage<'a,Force>,ReadStorage<'a,Mass>);

    fn run(&mut self, (mut force, mass): Self::SystemData) {
        for (mut force, mass) in (&mut force, &mass).join() {
            force.force =
                force.force + mass.value * constant::AMU * constant::GC * Vector3::new(0., 0., -1.);
        }
    }
}