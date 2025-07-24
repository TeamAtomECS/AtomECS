use bevy::prelude::*;
use crate::shapes::{
    Cylinder as MyCylinder,
    Cuboid as MyCuboid,
    Sphere as MySphere,
}; // Aliasing issues.
use crate::atom::{Atom, Position};
use crate::integrator::AtomECSBatchStrategy;
use nalgebra::Vector3;

/// Struct to signify shape is a wall
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Wall;

pub trait Intersect{
    /// Calculate intersection point between atom trajectory and shape and return point of intersection
    fn intersect(&self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        time: f64)
        -> Option<Vector3<f64>>;
}

pub trait Normal{
    /// Calculate normal at point of intersection (collision point)
    fn calculate_normal(&self, point: Vector3<f64>, wall_pos: Vector3<f64>) -> Vector3<f64>;
}

impl Intersect for MySphere{
    fn intersect(&self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        time: f64
    ) -> Option<Vector3<f64>> {
        let delta = atom_pos - wall_pos;
        let a = atom_vel.dot(atom_vel);
        let b = 2.0 * atom_vel.dot(&delta);
        let c = delta.dot(&delta) - &self.radius.powi(2);
        
        let discriminant = b.powi(2) - 4.0 * a * c;

        // Should never happen
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_discriminant = discriminant.sqrt();
        let t1 = (-b - sqrt_discriminant) / (2.0 * a);
        let t2 = (-b + sqrt_discriminant) / (2.0 * a);

        let t = if t1 >= 0.0 { t1 } else { t2 };
        
        // Should never happen
        if t > time || t < 0.0 {
            return None;
        }

        let intersection_point = atom_pos + t * atom_vel;
        Some(intersection_point)
    }
}

impl Intersect for MyCuboid{
    fn intersect(&self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        time: f64)
        -> Option<Vector3<f64>> {
            todo!("Implement intersection")
    }
}

impl Intersect for MyCylinder{
    fn intersect(&self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        time: f64)
        -> Option<Vector3<f64>> {
            todo!("Implement intersection")
    }
}

impl Normal for MySphere {
    fn calculate_normal(&self, point: Vector3<f64>, wall_pos: Vector3<f64>) -> Vector3<f64> {
        let normal_vector = point - wall_pos;
        normal_vector.normalize()
    }
}

impl Normal for MyCuboid{
    fn calculate_normal(&self, point: Vector3<f64>, wall_pos: Vector3<f64>) -> Vector3<f64> {
        todo!("Implement calculate normal")
    }
}

impl Normal for MyCylinder{
    fn calculate_normal(&self, point: Vector3<f64>, wall_pos: Vector3<f64>) -> Vector3<f64> {
        todo!("Implement calculate normal")
    }
}

/// Calculates collision point and normal
/// Should be run right after verlet integrate position
pub fn calculate_collision_info_system(
    query: Query<(&Position, &Velocity, &WallDistance, &OldWallDistance), With<Atom>>,
    sphere_walls: Query<(&Position, &MySphere), With<Wall>>,
    cuboid_walls: Query<(&Position, &MyCuboid), With<Wall>>,
    cylinder_walls: Query<(&Position, &MyCylinder), With<Wall>>,
    batch_strategy: Res<AtomECSBatchStrategy>, 
    timestep: Res<Timestep>,
) {
    // Makes no distinction between the positive (+1) and negative (-1) sides of two 
    // different wall entities. Could be an issue with multiple walls close to each other.
    dt = timestep.delta;

    query
        .par_iter()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|atom_pos, atom_vel, wall_distance, old_wall_distance| {
            if wall_distance*old_wall_distance == -1 { //collided

            for (wall_pos, sphere) in sphere_walls.iter() {
                collision_point = sphere.intersect(atom_pos, atom_vel, wall_pos, dt);
                normal = sphere.normal(collision_point.unwrap(), wall_pos);
            }

            for (wall_pos, cuboid) in cuboid_walls.iter() {
            }

            for (wall_pos, cylinder) in cylinder_walls.iter() {
            } 
        }
    });
}