#[cfg(test)]
pub mod tests {
    #[test]
    fn test_1D() {
        extern crate specs;
        use crate::atom::*;
        use crate::integrator::Step;

        use crate::initiate::NewlyCreated;
        use crate::simulation_templates::loadfromconfig::create_from_config;
        use assert_approx_eq::assert_approx_eq;
        use nalgebra::Vector3;
        use specs::{Builder, Entity, Join, RunNow, World};
        let (mut world, mut dispatcher) = create_from_config("test1D.yaml");

        for _i in 0..2000 {
            dispatcher.dispatch(&mut world.res);
            world.maintain();
        }
        let mut position = 0.;
        let pos_storage = world.read_storage::<Position>();
        let atom_storage = world.read_storage::<Atom>();
        for (_atom, pos) in (&atom_storage, &pos_storage).join() {
            position = pos.pos[2];
        }
        assert_approx_eq!(position, -0.0162, 0.0001);
    }
}