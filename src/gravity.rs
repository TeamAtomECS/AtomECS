//! Implements the force of gravity.

use crate::atom::{Force, Mass};
use crate::constant;
use crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME;
use crate::simulation::Plugin;
use nalgebra::Vector3;
use specs::prelude::*;

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

        match gravity_option {
            None => (),
            Some(_) => {
                (&mut force, &mass)
                    .par_join()
                    .for_each(|(force, mass)| {
                        force.force += mass.value * constant::AMU * constant::GC * Vector3::new(0., 0., -1.);
                    });
            }
        }
    }
}

/// This plugin implements the force of gravity.
/// 
/// See also [crate::gravity].
pub struct GravityPlugin;
impl Plugin for GravityPlugin {
    fn build(&self, builder: &mut crate::simulation::SimulationBuilder) {
        builder.dispatcher_builder.add(
            ApplyGravitationalForceSystem,
            "add_gravity",
            &["clear", INTEGRATE_POSITION_SYSTEM_NAME],
        );  
    }
    fn deps(&self) -> Vec::<Box<dyn Plugin>> {
        Vec::new()
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Tests the correct implementation of the `ApplyGravitationalForceSystem`
    #[test]
    fn test_apply_gravitational_force_system() {
        let mut test_world = World::new();

        test_world.register::<Mass>();
        test_world.register::<Force>();
        test_world.insert(ApplyGravityOption);

        let atom1 = test_world
            .create_entity()
            .with(Mass { value: 1.0 })
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();
        let mut system = ApplyGravitationalForceSystem;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();

        assert_approx_eq!(
            sampler_storage.get(atom1).expect("entity not found").force[2],
            -1.0 * constant::AMU * constant::GC,
            1e-30_f64
        );
    }
}
