//! Support for volumes to define simulation regions.
//!
//! This module tests entities to see if they should be deleted, based on their
//! position compared to any defined simulation volumes.

// This module assumes that all 'atoms' have the `RegionTestResult` attached.
// Perhaps there is some nice macro I can write to produce the required attachment systems?
// This pattern is also used elsewhere, eg `MagneticFieldSampler`.

use crate::atom::Position;
use crate::initiate::NewlyCreated;
use crate::integrator::BatchSize;
use crate::shapes::{Cuboid, Cylinder, Sphere, Volume};
use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;


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
#[derive(Component)]
struct RegionTest {
    result: Result,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SimulationVolume {
    pub volume_type: VolumeType,
}

/// Performs region tests for the defined volume type `T`.
///
/// For [VolumeType](struct.VolumeType.html)s that are `Inclusive`, the
/// test result is set to either `Failed` or `Accept`, depending on whether
/// the volume contains the entity. No entity is outright rejected.
///
/// For [VolumeType](struct.VolumeType.html)s that are `Exclusive`, the test
/// result is set to `Reject` if the volume contains the entity.
fn perform_region_tests<T: Volume + Component>(
    volume_query: Query<(&T, &SimulationVolume, &Position)>,
    mut atom_query: Query<(&mut RegionTest, &Position)>,
    batch_size: Res<BatchSize>,
    task_pool: Res<ComputeTaskPool>
) {
    for (volume, sim_volume, vol_pos) in volume_query.iter() {
        atom_query.par_for_each_mut(
            &task_pool,
            batch_size.0,
            |(mut result, pos)| {
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
        );
    }
}

/// This system sets all [RegionTest](struct.RegionTest.html) results
/// to the value `Result::Untested`.
fn clear_region_tests(
    mut query: Query<&mut RegionTest>,
    batch_size: Res<BatchSize>,
    task_pool: Res<ComputeTaskPool>
) {
    query.par_for_each_mut(
        &task_pool,
        batch_size.0,
        |mut test| {test.result = Result::Untested}
    );
}

/// This system deletes all entities with a [RegionTest](struct.RegionTest.html)
/// component with `Result::Reject` or `Result::Failed`.
fn delete_failed_region_tests(
    query: Query<(Entity, &RegionTest)>,
    mut commands: Commands
) {
    for (entity, test) in query.iter() {
        match test.result {
            Result::Reject | Result::Failed => {
                commands.entity(entity).despawn();
            }
            _ => (),
        }
    }
}

/// This sytem attaches [RegionTest](struct.RegionTest.html) components
/// to all entities that are [NewlyCreated](struct.NewlyCreated.html).
pub fn attach_region_tests_to_newly_created(
    query: Query<Entity, With<NewlyCreated>>,
    mut commands: Commands
) {
    for entity in query.iter() {
        commands.entity(entity).insert(
            RegionTest {
                result: Result::Untested,
            },
        );
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum SimRegionSystems {
    Set,
    ClearRegionTests,
    RegionTestVolume,
    DeleteRegionTestFailure,
    AttachRegionTestsToNewlyCreated
}

/// This plugin implements simulation bounds, and the removal of atoms which leave them.
/// 
/// See also [crate::sim_region]
#[derive(Default)]
pub struct SimulationRegionPlugin;
impl Plugin for SimulationRegionPlugin {
    fn build(&self, app: &mut App) {
        
        app.add_system_set(
            SystemSet::new().label(SimRegionSystems::Set)
            .with_system(clear_region_tests.label(SimRegionSystems::ClearRegionTests))
            .with_system(perform_region_tests::<Sphere>.label(SimRegionSystems::RegionTestVolume).after(SimRegionSystems::ClearRegionTests))
            .with_system(perform_region_tests::<Cuboid>.label(SimRegionSystems::RegionTestVolume).after(SimRegionSystems::ClearRegionTests))
            .with_system(perform_region_tests::<Cylinder>.label(SimRegionSystems::RegionTestVolume).after(SimRegionSystems::ClearRegionTests))
            .with_system(delete_failed_region_tests.label(SimRegionSystems::DeleteRegionTestFailure).after(SimRegionSystems::RegionTestVolume))
            .with_system(attach_region_tests_to_newly_created.label(SimRegionSystems::AttachRegionTestsToNewlyCreated))
        );
        app.init_resource::<BatchSize>();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::atom::Position;
    use nalgebra::Vector3;

    #[test]
    fn test_clear_region_tests_system() {
        let mut app = App::new();

        let tester = app.world.spawn()
            .insert(RegionTest {
                result: Result::Accept,
            })
            .id();

        app.add_plugin(SimulationRegionPlugin);
        app.update();

        let test = app.world.entity(tester).get::<RegionTest>().expect("Could not find entity");
        match test.result {
            Result::Untested => (),
            _ => panic!("Result not set to Result::Untested."),
        };
    }

    #[test]
    fn test_sphere_contains() {
        use rand;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut app = App::new();

        let sphere_pos = Vector3::new(1.0, 1.0, 1.0);
        let sphere_radius = 1.0;
        app.world.spawn()
            .insert(Position { pos: sphere_pos })
            .insert(Sphere {
                radius: sphere_radius,
            })
            .insert(SimulationVolume {
                volume_type: VolumeType::Inclusive,
            });

        // Create 100 entities at random positions. Save the expected value of their result.
        let mut tests = Vec::<(Entity, bool)>::new();
        for _ in 0..100 {
            let pos = Vector3::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
            );
            let entity = app.world.spawn()
                .insert(RegionTest {
                    result: Result::Untested,
                })
                .insert(Position { pos })
                .id();

            let delta = pos - sphere_pos;
            tests.push((entity, delta.norm_squared() < sphere_radius * sphere_radius));
        }

        app.add_system(perform_region_tests::<Sphere>);
        app.init_resource::<BatchSize>();
        app.update();
        
        for (entity, result) in tests {
            let test_result = app.world.entity(entity).get::<RegionTest>().expect("Could not find entity");
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
        let mut rng = rand::thread_rng();
        let mut app = App::new();

        let cuboid_pos = Vector3::new(1.0, 1.0, 1.0);
        let half_width = Vector3::new(0.2, 0.3, 0.1);
        app.world.spawn()
            .insert(Position { pos: cuboid_pos })
            .insert(Cuboid {
                half_width,
            })
            .insert(SimulationVolume {
                volume_type: VolumeType::Inclusive,
            });

        // Create 100 entities at random positions. Save the expected value of their result.
        let mut tests = Vec::<(Entity, bool)>::new();
        for _ in 0..100 {
            let pos = Vector3::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
            );
            let entity = app.world.spawn()
                .insert(RegionTest {
                    result: Result::Untested,
                })
                .insert(Position { pos })
                .id();

            let delta = pos - cuboid_pos;
            tests.push(
                (
                    entity,
                    delta[0].abs() < half_width[0]
                    && delta[1].abs() < half_width[1]
                    && delta[2].abs() < half_width[2],
                )
            );
        }

        app.add_system(perform_region_tests::<Cuboid>);
        app.init_resource::<BatchSize>();
        app.update();

        for (entity, result) in tests {
            let test_result = app.world.entity(entity).get::<RegionTest>().expect("Could not find entity");
            match test_result.result {
                Result::Failed => assert!(!result, "Incorrect Failed"),
                Result::Accept => assert!(result, "Incorrect Accept"),
                _ => panic!("Result must be either Failed or Accept"),
            }
        }
    }

    #[test]
    fn test_region_tests_are_added() {
        let mut app = App::new();
        app.add_plugin(SimulationRegionPlugin);
        let sampler_entity = app.world.spawn().insert(NewlyCreated).id();
        app.update();
        assert!(app.world.entity(sampler_entity).contains::<RegionTest>());
    }
}
