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


const MAX_STEPS: usize = 128; // Max steps for calculate_intersect function
const SURFACE_THRESHOLD: f64 = 1e-6; // Minimum for detecting a collision

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

#[derive(Component)]
pub struct DistanceToTravel {
    pub distance_to_travel: f64
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
    fn calculate_intersect(&self,
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

/// Credit to Inigo Quilez's for the SDF's. Found at https://www.shadertoy.com/view/Xds3zN

/// Signed distance to a sphere.
fn sd_sphere(point: &Vector3<f64>, radius: &f64) -> f64 {
    point.norm() - radius
}

/// Signed distance to an axis-aligned box centered at the origin.
fn sd_box(point: &Vector3<f64>, half_width: &Vector3<f64>) -> f64 {
    let delta = Vector3::new(
        point.x.abs() - half_width.x,
        point.y.abs() - half_width.y,
        point.z.abs() - half_width.z,
    );

    let outside = Vector3::new(
        delta.x.max(0.0),
        delta.y.max(0.0),
        delta.z.max(0.0),
    );

    let outside_dist = outside.norm();
    let inside_dist = f64::min(f64::max(delta.x, f64::max(delta.y, delta.z)), 0.0);

    outside_dist + inside_dist
}

/// Signed distance to a capped cylinder.
/// `cyl_start` and `cyl_end` define the axis of the cylinder (centre of the caps).
fn sd_cylinder(point: &Vector3<f64>, cyl_start: &Vector3<f64>, cyl_end: &Vector3<f64>, radius: &f64) -> f64 {
    // 
    let local_point = point - cyl_start;
    let axis = cyl_end - cyl_start;

    let axis_squared = axis.dot(&axis);
    let point_dot_axis = local_point.dot(&axis);

    let radial_dist = (local_point * axis_squared - axis * point_dot_axis).norm() - radius * axis_squared;

    let axial_dist = (point_dot_axis - axis_squared * 0.5).abs() - axis_squared * 0.5;

    let radial_sq = radial_dist * radial_dist;
    let axial_sq = axial_dist * axial_dist * axis_squared;

    let dist = if radial_dist.max(axial_dist) < 0.0 {
        -radial_sq.min(axial_sq)
    } else {
        (if radial_dist > 0.0 { radial_sq } else { 0.0 }) + (if axial_dist > 0.0 { axial_sq } else { 0.0 })
    };

    dist.signum() * dist.abs().sqrt() / axis_squared
}

// Uses -velocity as direction, ignores effect of acceleration.
impl Intersect for MySphere {
    fn calculate_intersect(
        &self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        max_time: f64
    ) -> Option<Vector3<f64>> {
        let speed = atom_vel.norm();
        if speed == 0.0 {
            return None;
        }

        let direction = -atom_vel / speed; // Going back in time
        let max_distance = speed * max_time;
        
        let mut distance_traveled = 0.0;

        for _ in 0..MAX_STEPS {
            let current_pos = atom_pos + direction * distance_traveled;
            let local_pos = current_pos - wall_pos;

            let distance_to_surface = sd_sphere(&local_pos, &self.radius);

            // Hit the surface
            if distance_to_surface < SURFACE_THRESHOLD {
                return Some(current_pos);
            }

            // If the step would exceed max distance, stop
            if distance_traveled + distance_to_surface > max_distance {
                return None;
            }

            distance_traveled += distance_to_surface;
        }

        None
    }
}

// Uses -velocity as direction, ignores effect of acceleration.
impl Intersect for MyCuboid {
    fn calculate_intersect(
        &self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        time: f64
    ) -> Option<Vector3<f64>> {
        let speed = atom_vel.norm();
        if speed == 0.0 {
            return None;
        }

        let direction = -atom_vel / speed;
        let max_distance = speed * time;

        let mut distance_traveled = 0.0;

        for _ in 0..MAX_STEPS {
            let current_pos = atom_pos + direction * distance_traveled;
            let local_pos = current_pos - wall_pos;

            let distance_to_surface = sd_box(&local_pos, &self.half_width);

            if distance_to_surface < SURFACE_THRESHOLD {
                return Some(current_pos);
            }

            if distance_traveled + distance_to_surface > max_distance {
                return None;
            }

            distance_traveled += distance_to_surface;
        }

        None
    }
}

// Uses -velocity as direction, ignores effect of acceleration.
impl Intersect for MyCylinder {
    fn calculate_intersect(
        &self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        max_time: f64,
    ) -> Option<Vector3<f64>> {
        let speed = atom_vel.norm();
        if speed == 0.0 {
            return None;
        }

        let direction = -atom_vel / speed;
        let max_distance = speed * max_time;
        let cyl_start = -self.length*self.direction*0.5;
        let cyl_end = self.length*self.direction*0.5;

        let mut distance_traveled = 0.0;

        for _ in 0..MAX_STEPS {
            let current_pos = atom_pos + direction * distance_traveled;
            let local_pos = current_pos - wall_pos;
            let distance_to_surface = sd_cylinder(&local_pos, &cyl_start, &cyl_end, &self.radius);

            if distance_to_surface < SURFACE_THRESHOLD {
                return Some(current_pos);
            }

            if distance_traveled + distance_to_surface > max_distance {
                return None;
            }

            distance_traveled += distance_to_surface;
        }

        None
    }
}

impl Normal for MySphere {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        let normal_vector = point - wall_pos;

        // Gives the normal assuming collision from the inside
        if (normal_vector.norm() - self.radius).abs() < SURFACE_THRESHOLD  {
            Some(-normal_vector.normalize())
        } else { None } // Should never happen
    }
}

impl Normal for MyCuboid {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        let local = point - wall_pos;

        // Gives the normal assuming collision from the inside
        if (local.x.abs() - self.half_width.x).abs() < SURFACE_THRESHOLD {
            Some(-Vector3::new(local.x.signum(), 0.0, 0.0))
        }
        else if (local.y.abs() - self.half_width.y).abs() < SURFACE_THRESHOLD{
            Some(-Vector3::new(0.0, local.y.signum(), 0.0))
        }
        else if (local.z.abs() - self.half_width.z).abs() < SURFACE_THRESHOLD{
            Some(-Vector3::new(0.0, 0.0, local.z.signum()))
        }
        else {
            None
        }
    }
}

impl Normal for MyCylinder {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        // Convert to local space
        let rel = point - wall_pos;
        let local = Vector3::new(
            rel.dot(&self.perp_x),
            rel.dot(&self.perp_y),
            rel.dot(&self.direction),
        );

        // Gives the normal assuming collision from the inside
        if (local.z.abs() - self.length*0.5).abs() < SURFACE_THRESHOLD {
            let normal = self.direction * local.z.signum();
            Some(-normal) 
        } else if ((local.x.powf(2.0) + local.y.powf(2.0)).sqrt() - self.radius) < SURFACE_THRESHOLD {
            let normal = self.perp_x * local.x + self.perp_y * local.y;
            Some(-normal.normalize()) 
        }
        else{
            None
        }
    }
}

/// Specular collision
fn specular(collision_point: Vector3<f64>, collision_normal: Vector3<f64>, velocity: Vector3<f64>) -> Vector3<f64> {
    todo!()
}

/// Diffuse collision
fn diffuse(collision_point: Vector3<f64>, collision_normal: Vector3<f64>, velocity: Vector3<f64>) -> Vector3<f64> {
    todo!()
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
                    if let Some(collision_point) = shape.calculate_intersect(&pos.pos, &vel.vel, &wall_pos.pos, dt) {
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