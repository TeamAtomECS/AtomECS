//! Support for volumes to define simulation regions.
//! 
//! This module tests entities to see if they should be deleted, based on their
//! position compared to any defined simulation volumes.

// This module assumes that all 'atoms' have the `RegionTestResult` attached.
// Perhaps there is some nice macro I can write to produce the required attachment systems?
// This pattern is also used elsewhere, eg `MagneticFieldSampler`.

use crate::atom::{Atom, Position};
use specs::{Component, Entities, Join, NullStorage, Read, ReadStorage, System,WriteStorage, VecStorage};
use std::marker::PhantomData;
use nalgebra::Vector3;

/// The [SimulationBounds](struct.SimulationBounds.html) is a resource that defines a bounding box encompassing the simulation region.
/// Atoms outside of the bounding box are deleted by the [DestroyOutOfBoundsAtomsSystem](struct.DestroyOutOfBoundAtomsSystem.html).
///
/// The simulation region is cubic. Atoms are deleted if any `abs(pos[i]) > half_width[i]`.
/// See [Position](struct.Position.html) for details of atom positions.
pub struct Cuboid {
    /// The dimension of the cuboid volume, from center to vertex (1,1,1).
    pub half_width: Vector3<f64>,
}

pub struct Sphere { 
    /// The radius of the spherical volume
    pub radius: f64,
}

enum VolumeType {
    /// Entities within the volume are accepted
    Inclusive,
    /// Entities outside the volume are accepted, entities within are rejected.
    Exclusive
}

trait Volume {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool;
    fn get_type(&self) -> VolumeType;
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
        for (&volume, &vol_pos) in (&volumes, &positions).join() {
            for (&mut result, &pos) in (&mut test_results, &positions).join() {
                match result.result {
                    Result::Reject => (),
                    _ => {
                         let contained = volume.contains(&vol_pos.pos, &pos.pos);
                         match volume.get_type() {
                             Inclusive => if contained { result.result = Result::Accept; } else { result.result = Result::Failed; },
                             Exclusive => if contained { result.result = Result::Reject; }
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
        for &mut test in (&mut tests).join() {
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
struct AttachRegionTestsToNewlyCreatedSystem {

}