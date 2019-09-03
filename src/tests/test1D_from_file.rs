#[cfg(test)]
pub mod tests {
    #[test]
    fn test_1d() {
        extern crate specs;
        use crate::atom::*;
        use crate::laser::force::NumberKick;

        use crate::initiate::NewlyCreated;
        use crate::simulation_templates::loadfromconfig::create_from_config;
        use assert_approx_eq::assert_approx_eq;
        use nalgebra::Vector3;


        use crate::detector::Detected;
        use crate::output::file_output::FileOutputMarker;

        use crate::destructor::SimulationBounds;
        use crate::laser::force::RandomWalkMarker;
        use crate::laser::repump::{Dark, RepumpLoss};
        use specs::{Builder, Join};

        use crate::atom_sources::oven::OvenVelocityCap;

        let (mut world, mut dispatcher) = create_from_config("test1D.yaml");
        world.register::<NewlyCreated>();
        world.add_resource(RandomWalkMarker { value: false });
        world.register::<Dark>();
        world.register::<NumberKick>();
        world.register::<Detected>();
        world.add_resource(SimulationBounds { half_width: Vector3::new(0.1,0.1,0.1) });
        world.add_resource(OvenVelocityCap { cap: 1000. });
        world.add_resource(RepumpLoss { proportion: 0.0 });
        world.add_resource(FileOutputMarker { value: false });
        world
            .create_entity()
            .with(Atom)
            .with(NewlyCreated)
            .with(AtomInfo::strontium())
            .with(Force::new())
            .with(Mass { value: 88.0 })
            .with(Position {
                pos: Vector3::new(0., 0., -0.15),
            })
            .with(Velocity {
                vel: Vector3::new(0., 0., 50.),
            })
            .with(NumberKick { value: 0 })
            .build();
        for _i in 0..2000 {
            dispatcher.dispatch(&mut world.res);
            world.maintain();
        }
        let mut position = 0.;
        let pos_storage = world.read_storage::<Position>();
        let atom_storage = world.read_storage::<Atom>();
        for (_atom, pos) in (&atom_storage, &pos_storage).join() {
            println!("detect position");
            position = pos.pos[2];
        }
        assert_approx_eq!(position as f64, -0.0162, 0.0001);
    }
}