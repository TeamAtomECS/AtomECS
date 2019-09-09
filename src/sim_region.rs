//! Support for volumes to define simulation regions.
//! 
//! This module tests entities to see if they should be deleted, based on their
//! position compared to any defined simulation volumes.

// This module assumes that all 'atoms' have the `RegionTestResult` attached.
// Perhaps there is some nice macro I can write to produce the required attachment systems?
// This pattern is also used elsewhere, eg `MagneticFieldSampler`.

use crate::atom::{Position};
use specs::{Component, Entities, Join, Read, ReadStorage, System,WriteStorage, VecStorage, LazyUpdate, DispatcherBuilder, World, HashMapStorage};
use crate::initiate::NewlyCreated;
use std::marker::PhantomData;
use nalgebra::Vector3;

trait Volume {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool;
    fn get_type(&self) -> &VolumeType;
}

pub enum VolumeType {
    /// Entities within the volume are accepted
    Inclusive,
    /// Entities outside the volume are accepted, entities within are rejected.
    Exclusive
}

/// A cuboid volume.
pub struct Cuboid {
    /// The dimension of the cuboid volume, from center to vertex (1,1,1).
    pub half_width: Vector3<f64>,
    /// Whether the volume is `Inclusive` or `Exclusive`.
    pub vol_type: VolumeType,
}
impl Volume for Cuboid { 
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool {
        let delta = entity_position - volume_position;
        let result = delta[0].abs() < self.half_width[0] && delta[1].abs() < self.half_width[1] && delta[2].abs() < self.half_width[2];
        result
    }

    fn get_type(&self) -> &VolumeType { &self.vol_type }
}
impl Component for Cuboid {
    type Storage = HashMapStorage<Self>;
}

/// A spherical volume.
pub struct Sphere { 
    /// The radius of the spherical volume.
    pub radius: f64,
    /// Whether the volume is `Inclusive` or `Exclusive`.
    pub vol_type: VolumeType,
}
impl Volume for Sphere { 
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool {
        let delta = entity_position - volume_position;
        let result = delta.norm_squared() < self.radius * self.radius;
        result
    }

    fn get_type(&self) -> &VolumeType { &self.vol_type }
}
impl Component for Sphere {
    type Storage = HashMapStorage<Self>;
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
    Reject
}

/// Component that marks an entity should be region tested.
struct RegionTest {
    result : Result,
}
impl Component for RegionTest {
    type Storage = VecStorage<Self>;
}

/// Performs region tests for the defined volume type `T`.
/// 
/// For [VolumeType](struct.VolumeType.html)s that are `Inclusive`, the
/// test result is set to either `Failed` or `Accept`, depending on whether
/// the volume contains the entity. No entity is outright rejected. 
/// 
/// For [VolumeType](struct.VolumeType.html)s that are `Exclusive`, the test
/// result is set to `Reject` if the volume contains the entity.
struct RegionTestSystem<T : Volume> {
    marker: PhantomData<T>,
}
impl<'a, T> System<'a> for RegionTestSystem<T>
where T:Volume+Component
{
    type SystemData = (ReadStorage<'a, T>, WriteStorage<'a, RegionTest>, ReadStorage<'a, Position>);

    fn run (&mut self, (volumes, mut test_results, positions) : Self::SystemData)
    {
        for (volume, vol_pos) in (&volumes, &positions).join() {
            for (result, pos) in (&mut test_results, &positions).join() {
                match result.result {
                    Result::Reject => (),
                    _ => {
                         let contained = volume.contains(&vol_pos.pos, &pos.pos);
                         match volume.get_type() {
                             VolumeType::Inclusive => if contained { result.result = Result::Accept; } else { result.result = Result::Failed; },
                             VolumeType::Exclusive => if contained { result.result = Result::Reject; }
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
    type SystemData = (WriteStorage<'a,RegionTest>);

    fn run(&mut self, mut tests: Self::SystemData) {
        for test in (&mut tests).join() {
            test.result = Result::Untested;
        }
    }
}

/// This system deletes all entities with a [RegionTest](struct.RegionTest.html)
/// component with `Result::Reject` or `Result::Failed`.
struct DeleteFailedRegionTestsSystem;
impl <'a> System<'a> for DeleteFailedRegionTestsSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, RegionTest>);

    fn run(&mut self, (ents, tests): Self::SystemData) {
        for (entity, test) in (&ents, &tests).join() {
            match test.result {
                Result::Reject | Result::Failed => ents.delete(entity).expect("Could not delete entity"),
                _ => ()
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
			updater.insert(ent, RegionTest { result: Result::Untested });
		}
	}
}



/// Adds the systems required by `sim_region` to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the `sim_region` systems run.
pub fn add_systems_to_dispatch(
	builder: DispatcherBuilder<'static, 'static>,
	deps: &[&str],
) -> DispatcherBuilder<'static, 'static> {
	builder
		.with(ClearRegionTestSystem, "clear_region_test", deps)
		.with(
			RegionTestSystem::<Sphere> { marker: PhantomData},
			"region_test_sphere",
			&["clear_region_test"],
		)
        .with(
			RegionTestSystem::<Cuboid> { marker: PhantomData},
			"region_test_cuboid",
			&["region_test_sphere"],
		)
        .with(
			DeleteFailedRegionTestsSystem,
			"delete_region_test_failure",
			&["region_test_cuboid"],
		)
        .with(
            AttachRegionTestsToNewlyCreatedSystem,
            "attach_region_tests_to_newly_created",
            deps
        )
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<Sphere>();
	world.register::<Cuboid>();
	world.register::<RegionTest>();
}