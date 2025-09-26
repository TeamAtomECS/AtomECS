pub mod atom_collisions;
pub mod wall_collisions;
pub mod spatial_grid;
pub mod wall_collision_info;

use bevy::prelude::*;
use std::fmt;
use crate::collisions::{atom_collisions::*, wall_collisions::*, spatial_grid::*};
use crate::atom::Atom;
use crate::integrator::IntegrationSet;
use crate::shapes::{
    Cylinder as MyCylinder,
    Cuboid as MyCuboid,
    Sphere as MySphere,
    CylindricalPipe,
};
use rand;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::Rng;

/// A resource that indicates that the simulation should apply atom collisions
#[derive(Resource)]
pub struct ApplyAtomCollisions(pub bool);

/// A resource that indicates that the simulation should apply wall collisions
#[derive(Resource)]
pub struct ApplyWallCollisions(pub bool);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CollisionsSet {
    Set,
    WallCollisionSystems,
    AtomCollisionSystems,
}

fn apply_atom_collisions(apply_atom_collision: Res<ApplyAtomCollisions>) -> bool {
    apply_atom_collision.0
}

fn apply_wall_collisions(apply_wall_collision: Res<ApplyWallCollisions>) -> bool {
    apply_wall_collision.0
}

#[derive(Component, Clone)]
pub struct NumberOfWallCollisions {
    pub value: i32
}

impl fmt::Display for NumberOfWallCollisions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

/// Initializes number of wall collisions component
pub fn init_number_of_collisions_system (
    mut query: Query<Entity, (With<Atom>, Without<NumberOfWallCollisions>)>,
    mut commands: Commands,
) {
    for atom_entity in query.iter_mut() {
        commands.entity(atom_entity).insert(NumberOfWallCollisions {
            value: 0
        });
    }
}

pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().insert_resource(ApplyAtomCollisions(true));
        app.world_mut().insert_resource(ApplyWallCollisions(true));
        app.world_mut().insert_resource(MaxSteps(1000));
        app.world_mut().insert_resource(SurfaceThreshold(1e-9));

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
        app.add_systems(PreUpdate, assign_location_status_system
            .in_set(CollisionsSet::WallCollisionSystems)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, init_number_of_collisions_system
            .in_set(CollisionsSet::WallCollisionSystems)
            .run_if(apply_wall_collisions));
        app.add_systems(PreUpdate, update_distance_to_travel_system
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
        app.add_systems(PreUpdate, wall_collision_system::<CylindricalPipe>
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