pub mod atom_collisions;
pub mod wall_collisions;
pub mod spatial_grid;

use bevy::prelude::*;
use crate::collisions::atom_collisions::*;
use crate::integrator::IntegrationSet;
use crate::collisions::wall_collisions::*;
use crate::collisions::spatial_grid::*;
use crate::shapes::{
    Cylinder as MyCylinder,
    Cuboid as MyCuboid,
    Sphere as MySphere,
};

use rand;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::Rng;

/// A resource that indicates that the simulation should apply atom collisions
#[derive(Resource)]
pub struct ApplyAtomCollisions{
    pub apply_atom_collision: bool,
}

/// A resource that indicates that the simulation should apply wall collisions
#[derive(Resource)]
pub struct ApplyWallCollisions{
    pub apply_wall_collision: bool,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CollisionsSet {
    Set,
    WallCollisionSystems,
    AtomCollisionSystems,
}

fn apply_atom_collisions(apply_atom_collision: Res<ApplyAtomCollisions>) -> bool {
    apply_atom_collision.apply_atom_collision
}

fn apply_wall_collisions(apply_wall_collision: Res<ApplyWallCollisions>) -> bool {
    apply_wall_collision.apply_wall_collision
}

pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().insert_resource(ApplyAtomCollisions {apply_atom_collision: true});
        app.world_mut().insert_resource(ApplyWallCollisions {apply_wall_collision: true});

        // Atom Collision Systems
        // Note that the atom collisions system must be applied after the velocity integrator or it will violate conservation of energy and cause heating
        app.add_systems(PostUpdate, init_boxid_system
            .in_set(CollisionsSet::AtomCollisionSystems)
            .after(IntegrationSet::EndIntegration)
            .run_if(apply_atom_collisions));
        app.add_systems(PostUpdate, assign_boxid_system
            .in_set(CollisionsSet::AtomCollisionSystems)
            .after(init_boxid_system)
            .run_if(apply_atom_collisions));
        app.add_systems(PostUpdate, apply_collisions_system
            .in_set(CollisionsSet::AtomCollisionSystems)
            .after(assign_boxid_system)
            .run_if(apply_atom_collisions));
        
        // Wall Collisions Systems
        app.add_systems(Startup, create_cosine_distribution
            .in_set(CollisionsSet::Set)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, init_distance_to_travel_system
            .in_set(CollisionsSet::WallCollisionSystems)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, reset_distance_to_travel_system
            .in_set(CollisionsSet::WallCollisionSystems)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, wall_collision_system::<MySphere>
            .in_set(CollisionsSet::WallCollisionSystems)
            .after(IntegrationSet::BeginIntegration)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, wall_collision_system::<MyCuboid>
            .in_set(CollisionsSet::WallCollisionSystems)
            .after(IntegrationSet::BeginIntegration)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, wall_collision_system::<MyCylinder>
            .in_set(CollisionsSet::WallCollisionSystems)
            .after(IntegrationSet::BeginIntegration)
            .run_if(apply_wall_collisions));
    }
}

/// A copy of weighted probability distribution, used for the Lambertian cosine distribution
#[derive(Resource)]
pub struct LambertianProbabilityDistribution {
    values: Vec<f64>,
    weighted_index: WeightedIndex<f64>,
}

impl LambertianProbabilityDistribution {
    pub fn new(values: Vec<f64>, weights: Vec<f64>) -> Self {
        LambertianProbabilityDistribution {
            values,
            weighted_index: WeightedIndex::new(&weights).unwrap(),
        }
    }
}

impl Distribution<f64> for LambertianProbabilityDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let index = self.weighted_index.sample(rng);
        self.values[index]
    }
}