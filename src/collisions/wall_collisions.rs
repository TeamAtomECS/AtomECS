use bevy::prelude::*;
use crate::shapes::{
    Cylinder as MyCylinder,
    Cuboid as MyCuboid,
    Sphere as MySphere,
    Volume
}; // Aliasing issues with bevy.
use crate::atom::{Atom, Position, Velocity};
use crate::integrator::{Timestep, AtomECSBatchStrategy};
use nalgebra::Vector3;
use approx::abs_diff_eq;
use std::sync::Mutex;

/// Stolen from sim_region.
pub enum VolumeType {
    /// Entities within the volume are accepted
    /// Accounts for collision from the inside
    Inclusive,
    /// Entities outside the volume are accepted, entities within are rejected.
    /// Accounts for collisions from the outside
    Exclusive,
}

/// Struct to signify shape is a wall
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Wall{
    pub volume_type: VolumeType,
}

/// Collision event
/// Not sure if events are the best tool here, as opposed to components use elsewhere in the code.
#[derive(Event)]
pub struct CollisionEvent{
    pub atom: Entity,
    pub wall: Entity,
    pub collision_point: Vector3<f64>,
    pub collision_normal: Vector3<f64>
}

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
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>>;
}

impl Intersect for MySphere{
    fn intersect(&self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        time: f64
    ) -> Option<Vector3<f64>> {
        // Calculate intersection
        let delta = atom_pos - wall_pos;
        let a = atom_vel.dot(atom_vel);
        let b = -2.0 * atom_vel.dot(&delta); // System is meant to run backwards i.e. calculate collision point after collision.
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
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        let normal_vector = point - wall_pos;
        if abs_diff_eq!(normal_vector.norm(), self.radius, epsilon = 1e-10) {
            Some(normal_vector.normalize())
        } else { None } // Should never happen
    }
}

impl Normal for MyCuboid{
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        todo!("Implement calculate normal")
    }
}

impl Normal for MyCylinder{
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        todo!("Implement calculate normal")
    }
}

/// Checks which atoms have collided by checking if they have exited the containing volume.
/// Almost the same as perform_region_tests in sim_region.rs
pub fn collision_check_system<T: Volume + Component + Intersect + Normal + Sync>(
    wall_query: Query<(Entity, &T, &Wall, &Position)>,
    atom_query: Query<(Entity, &Position, &Velocity), With<Atom>>,
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
    mut writer: EventWriter<CollisionEvent>,) {

    let dt = timestep.delta;
    let all_events = Mutex::new(Vec::new());

    for (wall, shape, wall_volume, wall_pos) in wall_query.iter() {
        atom_query
            .par_iter()
            .batching_strategy(batch_strategy.0.clone())
            .for_each_init(|| Vec::new(), |local_buffer, (atom, pos, vel)| {
                let contained = shape.contains(&wall_pos.pos, &pos.pos);
                let should_collide = match wall_volume.volume_type {
                    VolumeType::Inclusive => !contained,
                    VolumeType::Exclusive => contained,
                };

                if should_collide {
                    if let Some(collision_point) = shape.intersect(&pos.pos, &vel.vel, &wall_pos.pos, dt) {
                        if let Some(collision_normal) = shape.calculate_normal(&collision_point, &wall_pos.pos) {
                            let event = CollisionEvent {
                                atom,
                                wall,
                                collision_point,
                                collision_normal,
                            };

                            local_buffer.push(event);
                        } else {
                            eprintln!(
                                "Collision point found not on surface. Atom pos: {}, Wall pos: {}",
                                pos.pos, wall_pos.pos
                            );
                        }
                    } else {
                        eprintln!(
                            "Collision point not found for collided atom. Atom pos: {}, Wall pos: {}",
                            pos.pos, wall_pos.pos
                        );
                    }
                }
                let mut all = all_events.lock().unwrap();
                all.extend(local_buffer.drain(..));
            });
    }
    
    for event in all_events.into_inner().unwrap() {
        writer.write(event);
    }
}
