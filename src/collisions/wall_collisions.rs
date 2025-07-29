// Wall collision logic. Does not work with forces/accelertions.

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

/// Collision info
pub struct CollisionInfo{
    pub atom: Entity,
    pub wall: Entity,
    pub collision_point: Vector3<f64>,
    pub collision_normal: Vector3<f64>
}

pub trait SDF {
    /// Calculate signed distance
    fn signed_distance(&self, point: &Vector3<f64>) -> f64;
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

/// Credit to Inigo Quilez's for the SDFs. Found at https://www.shadertoy.com/view/Xds3zN

/// Signed distance to a sphere.
impl SDF for MySphere{
    fn signed_distance(&self, point: &Vector3<f64>) -> f64 {
        point.norm() - &self.radius
    }
}

/// Signed distance to an axis-aligned box centered at the origin.
impl SDF for MyCuboid{
    fn signed_distance(&self, point: &Vector3<f64>) -> f64 {
        let delta = Vector3::new(
            point.x.abs() - &self.half_width.x,
            point.y.abs() - &self.half_width.y,
            point.z.abs() - &self.half_width.z,
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
}

/// Signed distance to a capped cylinder.
/// `cyl_start` and `cyl_end` define the axis of the cylinder (centre of the caps).
impl SDF for MyCylinder{
    fn signed_distance(&self, point: &Vector3<f64>) -> f64 {
        let cyl_start = -self.direction*self.length*0.5;
        let local_point = point - cyl_start;

        let point_dot_axis = local_point.dot(&self.direction);

        let radial_dist = (local_point - self.direction * point_dot_axis).norm() - self.radius;

        let axial_dist = (point_dot_axis - self.length * 0.5).abs() - self.length * 0.5;

        let radial_sq = radial_dist * radial_dist;
        let axial_sq = axial_dist * axial_dist;

        let dist = if radial_dist.max(axial_dist) < 0.0 {
            -radial_sq.min(axial_sq)
        } else {
            (if radial_dist > 0.0 { radial_sq } else { 0.0 }) + (if axial_dist > 0.0 { axial_sq } else { 0.0 })
        };

        dist.signum() * dist.abs().sqrt()
    }
}

// Uses -velocity as direction, ignores effect of acceleration.
impl<T : SDF> Intersect for T {
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

        let mut distance_traveled = 0.0;

        for _ in 0..MAX_STEPS {
            let current_pos = atom_pos + direction * distance_traveled;
            let local_pos = current_pos - wall_pos;

            let distance_to_surface = self.signed_distance(&local_pos);

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

/// Specular reflection
fn specular(
    collision_normal: &Vector3<f64>,
    velocity: &Vector3<f64>,
) -> Vector3<f64> {
    let reflected = velocity - 2.0 * velocity.dot(collision_normal) * collision_normal;
    reflected
}


/// Diffuse collision
fn diffuse( 
    collision_normal: &Vector3<f64>, 
    velocity: &Vector3<f64>) -> Vector3<f64> {
    todo!()
}

/// Checks which atoms have collided by checking if they have exited the containing volume.
fn collision_check<T: Volume + Intersect + Normal>(
    atom: (Entity, &Position, &Velocity),
    wall: (Entity, &T, &Wall, &Position),        
    dt: f64,
) -> Option<CollisionInfo> {

    let (atom_entity, atom_pos, atom_vel) = atom;
    let (wall_entity, shape, wall_data, wall_pos) = wall;

    let contained = shape.contains(&wall_pos.pos, &atom_pos.pos);
    let should_collide = match wall_data.volume_type {
        VolumeType::Inclusive => !contained,
        VolumeType::Exclusive => contained,
    };

    if !should_collide {
        return None;
    }

    // Do collision check
    if let Some(collision_point) = shape.calculate_intersect(&atom_pos.pos, &atom_vel.vel, &wall_pos.pos, dt) {
        if let Some(collision_normal) = shape.calculate_normal(&collision_point, &wall_pos.pos) {

            return Some(CollisionInfo {
                atom: atom_entity,
                wall: wall_entity,
                collision_point,
                collision_normal,
            });
        } else {
            eprintln!(
                "Warning: Could not calculate normal. Atom: {:?}, Wall Pos: {:?}",
                atom_pos.pos, wall_pos.pos
            );
            None
        }
    } else {
        eprintln!(
            "Collision point not found for collided atom. Atom pos: {}, Wall pos: {}",
            atom_pos.pos, wall_pos.pos
        );
        None
    }
}

/// Do wall collision
fn do_wall_collision(
    atom: (&mut Position, &mut Velocity, &mut DistanceToTravel),
    collision: &CollisionInfo,
    dt: f64,
) -> f64 {
    let (mut atom_pos, mut atom_vel, mut distance) = atom;

    // subtract distance traveled to the collision point from previous position
    let traveled = ((atom_pos.pos - atom_vel.vel*dt) - collision.collision_point).norm();
    distance.distance_to_travel -= traveled;
    if distance.distance_to_travel < 0.0 {
        distance.distance_to_travel = 0.0;
        eprintln!("Distance to travel set to a negative value somehow")
    }

    // do collision
    atom_vel.vel = specular(&collision.collision_normal, &atom_vel.vel);

    if distance.distance_to_travel > 0.0 {
        // propagate along chosen direction
        atom_pos.pos = collision.collision_point + atom_vel.vel.normalize() * distance.distance_to_travel;
    } else {
        atom_pos.pos = collision.collision_point;
    }

    //Return time used
    traveled / atom_vel.vel.norm()
}

/// Initializes distance to travel component
pub fn init_distance_to_travel_system (
    mut query: Query<(Entity, &Velocity), (With<Atom>, Without<DistanceToTravel>)>,
    mut commands: Commands,
    timestep: Res<Timestep>,
) {
    for (atom_entity, vel) in query.iter_mut() {
        commands.entity(atom_entity).insert(DistanceToTravel {
            distance_to_travel: vel.vel.norm() * timestep.delta
        });
    }

}

/// Do the collisions
/// To be run after verlet integrate position
pub fn wall_collision_system<T: Volume + Component + Intersect + Normal>(
    wall_query: Query<(Entity, &T, &Wall, &Position)>,
    mut atom_query: Query<(Entity, &mut Position, &mut Velocity, &mut DistanceToTravel), With<Atom>>,
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
) {
    let dt = timestep.delta;
    let walls: Vec<_> = wall_query.iter().collect();

    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(atom, mut pos, mut vel, mut distance)| {
            let mut dt_atom = dt;
            while distance.distance_to_travel > 0.0 { 
                let mut collided = false;

                for (wall, shape, wall_volume, wall_pos) in &walls {
                    if let Some(collision_info) = collision_check(
                        (atom, &pos, &vel),
                        (*wall, *shape, *wall_volume, *wall_pos),
                        dt_atom,
                    ) {
                        // Handle the collision
                        let time_used = do_wall_collision((&mut pos, &mut vel, &mut distance), &collision_info, dt_atom);
                        collided = true;
                        dt_atom -= time_used;
                        if dt_atom < 0.0 {
                            dt_atom = 0.0;
                            eprintln!("Time set to a negative value somehow, wall collision system")
                        }
                    }
                }

                if !collided {
                    // No collision detected with any wall, so no more distance to travel
                    distance.distance_to_travel = 0.0;
                    break;
                }
            }
        });
}

/// Calculate distance to travel at the end of every frame
/// To be run after verlet integrate velocity
pub fn reset_distance_to_travel_system(
    mut query: Query<(&Velocity, &mut DistanceToTravel), With<Atom>>,
    timestep: Res<Timestep>,
) {
    let dt = timestep.delta;

    for (vel, mut distance) in query.iter_mut() {
        distance.distance_to_travel = vel.vel.norm() * dt;
    }
}

