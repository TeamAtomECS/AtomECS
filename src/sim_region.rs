//! Support for volumes to define simulation regions.
//!
//! This module tests entities to see if they should be deleted, based on their
//! position compared to any defined simulation volumes.

// This module assumes that all 'atoms' have the `RegionTestResult` attached.
// Perhaps there is some nice macro I can write to produce the required attachment systems?
// This pattern is also used elsewhere, eg `MagneticFieldSampler`.

use crate::atom::Position;
use crate::initiate::NewlyCreated;
use crate::shapes::{Cuboid, Cylinder, Sphere, Volume};
use crate::simulation::Plugin;
use specs::prelude::*;
use std::marker::PhantomData;

pub enum VolumeType {
    /// Entities within the volume are accepted
    Inclusive,
    /// Entities outside the volume are accepted, entities within are rejected.
    Exclusive,
}

/// All possible results of region testing.
enum Result {
    /// The entity has not yet been tested
    Untested,
    /// The entity has been tested and failed at least once, but has not yet been outright rejected.
    Failed,
    /// The entity has been accepted _so far_.
    Accept,
    /// The entity is outright rejected.
    Reject,
}

/// Component that marks an entity should be region tested.
struct RegionTest {
    result: Result,
}
impl Component for RegionTest {
    type Storage = VecStorage<Self>;
}

pub struct SimulationVolume {
    pub volume_type: VolumeType,
}
impl Component for SimulationVolume {
    type Storage = HashMapStorage<Self>;
}

/// Performs region tests for the defined volume type `T`.
///
/// For [VolumeType](struct.VolumeType.html)s that are `Inclusive`, the
/// test result is set to either `Failed` or `Accept`, depending on whether
/// the volume contains the entity. No entity is outright rejected.
///
/// For [VolumeType](struct.VolumeType.html)s that are `Exclusive`, the test
/// result is set to `Reject` if the volume contains the entity.
struct RegionTestSystem<T: Volume> {
    marker: PhantomData<T>,
}
impl<'a, T> System<'a> for RegionTestSystem<T>
where
    T: Volume + Component,
{
    type SystemData = (
        ReadStorage<'a, T>,
        ReadStorage<'a, SimulationVolume>,
        WriteStorage<'a, RegionTest>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (volumes, sim_volumes, mut test_results, positions): Self::SystemData) {
        for (volume, sim_volume, vol_pos) in (&volumes, &sim_volumes, &positions).join() {
            for (result, pos) in (&mut test_results, &positions).join() {
                match result.result {
                    Result::Reject => (),
                    _ => {
                        let contained = volume.contains(&vol_pos.pos, &pos.pos);
                        match sim_volume.volume_type {
                            VolumeType::Inclusive => {
                                if contained {
                                    result.result = Result::Accept;
                                } else if let Result::Untested = result.result {
                                    result.result = Result::Failed
                                }
                            }
                            VolumeType::Exclusive => {
                                if contained {
                                    result.result = Result::Reject;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// This system sets all [RegionTest](struct.RegionTest.html) results
/// to the value `Result::Untested`.
struct ClearRegionTestSystem;
impl<'a> System<'a> for ClearRegionTestSystem {
    type SystemData = WriteStorage<'a, RegionTest>;

    fn run(&mut self, mut tests: Self::SystemData) {
        for test in (&mut tests).join() {
            test.result = Result::Untested;
        }
    }
}

/// This system deletes all entities with a [RegionTest](struct.RegionTest.html)
/// component with `Result::Reject` or `Result::Failed`.
struct DeleteFailedRegionTestsSystem;
impl<'a> System<'a> for DeleteFailedRegionTestsSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, RegionTest>);

    fn run(&mut self, (ents, tests): Self::SystemData) {
        for (entity, test) in (&ents, &tests).join() {
            match test.result {
                Result::Reject | Result::Failed => {
                    ents.delete(entity).expect("Could not delete entity")
                }
                _ => (),
            }
        }
    }
}

/// This sytem attaches [RegionTest](struct.RegionTest.html) components
/// to all entities that are [NewlyCreated](struct.NewlyCreated.html).
struct AttachRegionTestsToNewlyCreatedSystem;
impl<'a> System<'a> for AttachRegionTestsToNewlyCreatedSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, NewlyCreated>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
        for (ent, _nc) in (&ent, &newly_created).join() {
            updater.insert(
                ent,
                RegionTest {
                    result: Result::Untested,
                },
            );
        }
    }
}

/// This plugin implements simulation bounds, and the removal of atoms which leave them.
/// 
/// See also [crate::sim_region]
#[derive(Default)]
pub struct SimulationRegionPlugin;
impl Plugin for SimulationRegionPlugin {
    fn build(&self, builder: &mut crate::simulation::SimulationBuilder) {
        add_systems_to_dispatch(&mut builder.dispatcher_builder, &[]);
        register_components(&mut builder.world);
    }
    fn deps(&self) -> Vec::<Box<dyn Plugin>> {
        Vec::new()
    }
}

/// Adds the systems required by `sim_region` to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the `sim_region` systems run.
fn add_systems_to_dispatch(
    builder: &mut DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) {
    builder.add(ClearRegionTestSystem, "clear_region_test", deps);
    builder.add(
        RegionTestSystem::<Sphere> {
            marker: PhantomData,
        },
        "region_test_sphere",
        &["clear_region_test"],
    );
    builder.add(
        RegionTestSystem::<Cuboid> {
            marker: PhantomData,
        },
        "region_test_cuboid",
        &["region_test_sphere"],
    );
    builder.add(
        RegionTestSystem::<Cylinder> {
            marker: PhantomData,
        },
        "region_test_cylinder",
        &["region_test_cuboid"],
    );
    builder.add(
        DeleteFailedRegionTestsSystem,
        "delete_region_test_failure",
        &["region_test_cylinder"],
    );
    builder.add(
        AttachRegionTestsToNewlyCreatedSystem,
        "attach_region_tests_to_newly_created",
        deps,
    );
}

/// Registers resources required by magnetics to the ecs world.
fn register_components(world: &mut World) {
    world.register::<Sphere>();
    world.register::<Cuboid>();
    world.register::<Cylinder>();
    world.register::<SimulationVolume>();
    world.register::<RegionTest>();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::atom::Position;
    use nalgebra::Vector3;
    use specs::{Builder, DispatcherBuilder, RunNow, World};

    #[test]
    fn test_clear_region_tests_system() {
        let mut test_world = World::new();
        register_components(&mut test_world);

        let tester = test_world
            .create_entity()
            .with(RegionTest {
                result: Result::Accept,
            })
            .build();

        let mut system = ClearRegionTestSystem {};
        system.run_now(&test_world);

        let tests = test_world.read_storage::<RegionTest>();
        let test = tests.get(tester).expect("Could not find entity");
        match test.result {
            Result::Untested => (),
            _ => panic!("Result not set to Result::Untested."),
        };
    }

    #[test]
    fn test_sphere_contains() {
        use rand;
        use rand::Rng;
        use specs::Entity;
        let mut rng = rand::thread_rng();

        let mut test_world = World::new();
        register_components(&mut test_world);
        test_world.register::<Position>();

        let sphere_pos = Vector3::new(1.0, 1.0, 1.0);
        let sphere_radius = 1.0;
        test_world
            .create_entity()
            .with(Position { pos: sphere_pos })
            .with(Sphere {
                radius: sphere_radius,
            })
            .with(SimulationVolume {
                volume_type: VolumeType::Inclusive,
            })
            .build();

        // Create 100 entities at random positions. Save the expected value of their result.
        let mut tests = Vec::<(Entity, bool)>::new();
        for _ in 1..100 {
            let pos = Vector3::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
            );
            let entity = test_world
                .create_entity()
                .with(RegionTest {
                    result: Result::Untested,
                })
                .with(Position { pos })
                .build();

            let delta = pos - sphere_pos;
            tests.push((entity, delta.norm_squared() < sphere_radius * sphere_radius));
        }

        let mut system = RegionTestSystem::<Sphere> {
            marker: PhantomData,
        };
        system.run_now(&test_world);

        let test_results = test_world.read_storage::<RegionTest>();
        for (entity, result) in tests {
            let test_result = test_results.get(entity).expect("Could not find entity");
            match test_result.result {
                Result::Failed => assert!(!result, "Incorrect Failed"),
                Result::Accept => assert!(result, "Incorrect Accept"),
                _ => panic!("Result must be either Failed or Accept"),
            }
        }
    }

    #[test]
    fn test_cuboid_contains() {
        use rand;
        use rand::Rng;
        use specs::Entity;
        let mut rng = rand::thread_rng();

        let mut test_world = World::new();
        register_components(&mut test_world);
        test_world.register::<Position>();

        let cuboid_pos = Vector3::new(1.0, 1.0, 1.0);
        let half_width = Vector3::new(0.2, 0.3, 0.1);
        test_world
            .create_entity()
            .with(Position { pos: cuboid_pos })
            .with(Cuboid {
                half_width,
            })
            .with(SimulationVolume {
                volume_type: VolumeType::Inclusive,
            })
            .build();

        // Create 100 entities at random positions. Save the expected value of their result.
        let mut tests = Vec::<(Entity, bool)>::new();
        for _ in 1..100 {
            let pos = Vector3::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
            );
            let entity = test_world
                .create_entity()
                .with(RegionTest {
                    result: Result::Untested,
                })
                .with(Position { pos })
                .build();

            let delta = pos - cuboid_pos;
            tests.push((
                entity,
                delta[0].abs() < half_width[0]
                    && delta[1].abs() < half_width[1]
                    && delta[2].abs() < half_width[2],
            ));
        }

        let mut system = RegionTestSystem::<Cuboid> {
            marker: PhantomData,
        };
        system.run_now(&test_world);

        let test_results = test_world.read_storage::<RegionTest>();
        for (entity, result) in tests {
            let test_result = test_results.get(entity).expect("Could not find entity");
            match test_result.result {
                Result::Failed => assert!(!result, "Incorrect Failed"),
                Result::Accept => assert!(result, "Incorrect Accept"),
                _ => panic!("Result must be either Failed or Accept"),
            }
        }
    }

    #[test]
    fn test_region_tests_are_added() {
        let mut test_world = World::new();
        register_components(&mut test_world);
        test_world.register::<NewlyCreated>();
        let mut builder = DispatcherBuilder::new();
        add_systems_to_dispatch(&mut builder, &[]);
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut test_world);

        let sampler_entity = test_world.create_entity().with(NewlyCreated).build();

        dispatcher.dispatch(&test_world);
        test_world.maintain();

        let samplers = test_world.read_storage::<RegionTest>();
        assert!(samplers.contains(sampler_entity));
    }
}
