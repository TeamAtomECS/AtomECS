/// Wall collision logic. Does not work with forces/accelertions, or multiple entities.

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

#[derive(Resource)]
pub struct MaxSteps(i32);

#[derive(Resource)]
pub struct SurfaceThreshold(f64); // Minimum penetration for detecting a collision

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

/// Collision infoconst SURFACE_THRESHOLD: f64 =  1e-9; // Minimum for detecting a collision
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
        time: f64,
        tolerance: f64,
        max_steps: i32,)
        -> Option<Vector3<f64>>;
}

pub trait Normal{
    /// Calculate normal at point of intersection (collision point)
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>>;
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
        tolerance: f64,
        max_steps: i32,
    ) -> Option<Vector3<f64>> {
        let speed = atom_vel.norm();
        if speed == 0.0 {
            println!("Speed 0 issue");
            return None;
        }

        let direction = -atom_vel / speed;
        let max_distance = speed * max_time;

        let mut distance_traveled = 0.0;

        for _ in 0..max_steps {
            let current_pos = atom_pos + direction * distance_traveled;
            let local_pos = current_pos - wall_pos;

            let distance_to_surface = self.signed_distance(&local_pos);

            if distance_to_surface < tolerance {
                return Some(current_pos);
            }

            if distance_traveled + distance_to_surface > max_distance {
                println!("Not enough time issue");
                return None;
            }

            distance_traveled += distance_to_surface;
        }
        println!("Not enough steps issue");
        None
    }
}

impl Normal for MySphere {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>> {
        let normal_vector = point - wall_pos;

        // Gives the normal assuming collision from the inside
        if (normal_vector.norm() - self.radius).abs() < tolerance  {
            Some(-normal_vector.normalize())
        } else { None } // Should never happen
    }
}

impl Normal for MyCuboid {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>> {
        let local = point - wall_pos;

        // Gives the normal assuming collision from the inside
        if (local.x.abs() - self.half_width.x).abs() < tolerance {
            Some(-Vector3::new(local.x.signum(), 0.0, 0.0))
        }
        else if (local.y.abs() - self.half_width.y).abs() < tolerance{
            Some(-Vector3::new(0.0, local.y.signum(), 0.0))
        }
        else if (local.z.abs() - self.half_width.z).abs() < tolerance{
            Some(-Vector3::new(0.0, 0.0, local.z.signum()))
        }
        else {
            None
        }
    }
}

impl Normal for MyCylinder {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>> {
        // Convert to local space
        let rel = point - wall_pos;
        let local = Vector3::new(
            rel.dot(&self.perp_x),
            rel.dot(&self.perp_y),
            rel.dot(&self.direction),
        );

        // Gives the normal assuming collision from the inside
        if (local.z.abs() - self.length*0.5).abs() < tolerance {
            let normal = self.direction * local.z.signum();
            Some(-normal) 
        } else if ((local.x.powf(2.0) + local.y.powf(2.0)).sqrt() - self.radius) < tolerance {
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


/// Diffuse collision (Lambertian)
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
    tolerance: f64, 
    max_steps: i32,
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
    if let Some(collision_point) = shape.calculate_intersect(&atom_pos.pos, &atom_vel.vel, &wall_pos.pos, dt, tolerance, max_steps) {
        if let Some(collision_normal) = shape.calculate_normal(&collision_point, &wall_pos.pos, tolerance) {

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
    threshold: Res<SurfaceThreshold>, 
    max_steps: Res<MaxSteps>,
) {
    let tolerance = threshold.0;
    let max_steps = max_steps.0;
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
                        tolerance,
                        max_steps,
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use assert_approx_eq::assert_approx_eq;

    const TEST_RUNS: usize = 10000;
    const TOLERANCE: f64 = 1e-6;
    const MAX_DT: f64 = 1e3;
    const MAX_STEPS: i32 = 1000;

    fn rand_vec(min: f64, max: f64) -> Vector3<f64> {
        let mut rng = rand::rng();
        Vector3::new(
            rng.random_range(min..=max),
            rng.random_range(min..=max),
            rng.random_range(min..=max),
        )
    }

    #[test]
    fn test_intersect_cuboid() {
        let mut success = 0;
        let mut failed = 0;
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let length = 1.0;
            let breadth = 1.5;
            let height = 2.0;
            let half_width = Vector3::new(length, breadth, height);

            let cuboid = MyCuboid { half_width };
            let wall_pos = rand_vec(-5.0, 5.0);

            let inside_local = Vector3::new(
                rng.random_range(-half_width.x ..=half_width.x),
                rng.random_range(-half_width.y ..=half_width.y),
                rng.random_range(-half_width.z..=half_width.z),
            );

            let x_out = if rng.random_bool(0.5) {
                rng.random_range(-2.0 * half_width.x..-half_width.x)
            } else {
                rng.random_range(half_width.x..2.0 * half_width.x)
            };

            let y_out = if rng.random_bool(0.5) {
                rng.random_range(-2.0 * half_width.y..-half_width.y)
            } else {
                rng.random_range(half_width.y..2.0 * half_width.y)
            };

            let z_out = if rng.random_bool(0.5) {
                rng.random_range(-2.0 * half_width.z..-half_width.z)
            } else {
                rng.random_range(half_width.z..2.0 * half_width.z)
            };

            let outside_local = Vector3::new(x_out, y_out, z_out);

            let inside = wall_pos + inside_local;
            let outside = wall_pos + outside_local;

            if cuboid.contains(&wall_pos, &outside) || !cuboid.contains(&wall_pos, &inside) {
                panic!("Wrong initialization in test");
            }

            let velocity = (outside - inside).normalize();
            let result = cuboid.calculate_intersect(&outside, &velocity, &wall_pos, MAX_DT, TOLERANCE, MAX_STEPS);

            match result {
                Some(hit) => {
                    let sdf_val = cuboid.signed_distance(&(hit - wall_pos));
                    if sdf_val.abs() < TOLERANCE {
                        success += 1;
                    } else {
                        panic!("[Cuboid] Intersection inaccurate: SDF = {}", sdf_val)
                    }
                }
                None => {
                    failed += 1;
                }
            }
        }

        let failure_rate = (failed as f64 / TEST_RUNS as f64) * 100.0;

        println!(
            "[Cuboid Test] Success: {}, Failed: {}, Failure Rate: {:.2}%",
            success,
            failed,
            failure_rate,
        );

        if failure_rate > 1.0 {
            panic!("Failure rate too high! More than 1%")
        }
    }

    #[test]
    fn test_intersect_sphere() {
        let mut success = 0;
        let mut failed = 0;
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let radius = rng.random_range(0.0..10.0);
            let sphere = MySphere { radius };
            let wall_pos = rand_vec(-5.0, 5.0);

            let radius_in = rng.random_range(0.0..radius);
            let inside_local = rand_vec(-1.0,1.0).normalize() * radius_in;

            let radius_out = rng.random_range(radius..radius*2.0);
            let outside_local = rand_vec(-1.0,1.0).normalize() * radius_out;

            let inside = wall_pos + inside_local;
            let outside = wall_pos + outside_local;

            if sphere.contains(&wall_pos, &outside) || !sphere.contains(&wall_pos, &inside) {
                panic!("Wrong initialization in test");
            }

            let velocity = (outside - inside).normalize();

            let result = sphere.calculate_intersect(&outside, &velocity, &wall_pos, MAX_DT, TOLERANCE, MAX_STEPS);

            match result {
                Some(hit) => {
                    let sdf_val = sphere.signed_distance(&(hit - wall_pos));
                    if sdf_val.abs() < TOLERANCE {
                        success += 1;
                    } else {
                        panic!("[Sphere] Intersection inaccurate: SDF = {}", sdf_val)
                    }
                }
                None => {
                    failed += 1;
                }
            }
        }

        let failure_rate = (failed as f64 / TEST_RUNS as f64) * 100.0;

        println!(
            "[Sphere Test] Success: {}, Failed: {}, Failure Rate: {:.2}%",
            success,
            failed,
            failure_rate,
        );

        if failure_rate > 1.0 {
            panic!("Failure rate too high! More than 1%")
        }

    }

    #[test]
    fn test_intersect_cylinder() {
        let mut success = 0;
        let mut failed = 0;
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let radius = rng.random_range(0.1..10.0);
            let length = rng.random_range(1.0..10.0);

            // Random direction vector
            let random_dir;
            random_dir = rand_vec(-1.0, 1.0);

            let wall_pos = rand_vec(-5.0, 5.0);
            let cylinder = MyCylinder::new(radius, length, random_dir);

            let (inside_local, outside_local) = {
                // Inside point
                let radial_in = rng.random_range(0.0..radius);
                let angle_in = rng.random_range(0.0..std::f64::consts::TAU);
                let x_in = radial_in * angle_in.cos();
                let y_in = radial_in * angle_in.sin();
                let z_in = rng.random_range(-length * 0.4..length * 0.4);
                let inside = cylinder.perp_x * x_in + cylinder.perp_y * y_in + cylinder.direction * z_in;
            
                // Outside point
                let radial_out = rng.random_range(radius * 1.1..radius * 2.0);
                let angle_out = rng.random_range(0.0..std::f64::consts::TAU);
                let x_out = radial_out * angle_out.cos();
                let y_out = radial_out * angle_out.sin();
                let z_out = if rng.random_bool(0.5) {
                    rng.random_range(-2.0 * length..-length*0.5)
                } else {
                    rng.random_range(0.5 * length..2.0 * length)
                };
                let outside = cylinder.perp_x * x_out + cylinder.perp_y * y_out + cylinder.direction * z_out;
            
                (inside, outside)
            };

            let inside = wall_pos + inside_local;
            let outside = wall_pos + outside_local;

            if cylinder.contains(&wall_pos, &outside) || !cylinder.contains(&wall_pos, &inside) {
                panic!("Wrong initialization in test");
            }

            let velocity = (outside - inside).normalize();

            let result = cylinder.calculate_intersect(&outside, &velocity, &wall_pos, MAX_DT, TOLERANCE, MAX_STEPS);

            match result {
                Some(hit) => {
                    let sdf_val = cylinder.signed_distance(&(hit - wall_pos));
                    if sdf_val.abs() < TOLERANCE {
                        success += 1;
                    } else {
                        panic!("[Cylinder] Intersection inaccurate: SDF = {}", sdf_val)
                    }
                }
                None => {
                    failed += 1;
                }
            }
        }

        let failure_rate = (failed as f64 / TEST_RUNS as f64) * 100.0;

        println!(
            "[Cylinder Test] Success: {}, Failed: {}, Failure Rate: {:.2}%",
            success,
            failed,
            failure_rate,
        );

        if failure_rate > 1.0 {
            panic!("Failure rate too high! More than 1%")
        }
    }

    #[test]
    fn test_normal_sphere() {
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let radius = rng.random_range(0.1..10.0);
            let sphere = MySphere { radius };
            let wall_pos = rand_vec(-5.0, 5.0);
            
            let inside_local = rand_vec(-1.0, 1.0).normalize() * radius;
            let inside = wall_pos + inside_local;

            let result = sphere.calculate_normal(&inside, &wall_pos, TOLERANCE);
            
            match result {
                Some(normal) => {
                    assert_approx_eq!(-inside_local.normalize()[0], normal[0], 1e-12);
                    assert_approx_eq!(-inside_local.normalize()[1], normal[1], 1e-12);
                    assert_approx_eq!(-inside_local.normalize()[2], normal[2], 1e-12);
                }
                None => panic!("Normal calculation failed for sphere"),
            }
        }
    }

    #[test]
    fn test_normal_cuboid() {
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let length = rng.random_range(0.0..10.0);
            let breadth = rng.random_range(0.0..10.0);
            let height = rng.random_range(0.0..10.0);
            let half_width = Vector3::new(length, breadth, height);

            let cuboid = MyCuboid { half_width };
            let wall_pos = rand_vec(-5.0, 5.0);
        
            let face = rand::random::<u8>() % 6; 
        
            let (inside_local, expected_normal) = match face {
                0 => (Vector3::new(length, rng.random_range(-breadth.. breadth), rng.random_range(-height..height)), Vector3::new(-1.0,0.0,0.0)),// Positive X face
                1 => (Vector3::new(-length, rng.random_range(-breadth.. breadth), rng.random_range(-height..height)),Vector3::new(1.0,0.0,0.0)), // Negative X face
                2 => (Vector3::new(rng.random_range(-length..length), breadth, rng.random_range(-height..height)), Vector3::new(0.0,-1.0,0.0)),// Positive Y face
                3 => (Vector3::new(rng.random_range(-length..length), -breadth, rng.random_range(-height..height)),Vector3::new(0.0,1.0,0.0)), // Negative Y face
                4 => (Vector3::new(rng.random_range(-length..length), rng.random_range(-breadth.. breadth), height), Vector3::new(0.0,0.0,-1.0)),// Positive Z face
                5 => (Vector3::new(rng.random_range(-length..length), rng.random_range(-breadth.. breadth), -height),Vector3::new(0.0,0.0,1.0)), // Negative Z face
                _ => panic!("Invalid face selection"), // This should never happen
            };
        
            let inside = wall_pos + inside_local;
        
            let result = cuboid.calculate_normal(&inside, &wall_pos, TOLERANCE);
        
            match result {
                Some(normal) => {
                    assert_approx_eq!(normal[0], expected_normal[0], 1e-12);
                    assert_approx_eq!(normal[1], expected_normal[1], 1e-12);
                    assert_approx_eq!(normal[2], expected_normal[2], 1e-12);
                }
                None => panic!("Normal calculation failed for cuboid"),
            }
        }
    }
    
    #[test]
    fn test_normal_cylinder() {
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let radius = 1.0;
            let length = 2.0;
            let random_dir = Vector3::new(0.0,0.0,1.0); // Random direction
            let cylinder = MyCylinder::new(radius, length, random_dir);
            let wall_pos = rand_vec(-5.0, 5.0);
    
            let face = rand::random::<u8>() % 3;
    
            let (surface_local, expected_normal) = match face {
                0 => {  // Curved face
                    let angle = rng.random_range(0.0..std::f64::consts::TAU);
                    let x = radius * angle.cos();
                    let y = radius * angle.sin();
                    let z = rng.random_range(-length*0.5..length*0.5);
                    let surface_local = x * cylinder.perp_x + y * cylinder.perp_y + z * cylinder.direction;
                    let normal = -(x * cylinder.perp_x + y * cylinder.perp_y).normalize();
                    (surface_local, normal) 
                },
                1 => {  // Top cap
                    let radial = rng.random_range(0.0..radius);
                    let angle = rng.random_range(0.0..std::f64::consts::TAU);
                    let x = radial * angle.cos();
                    let y = radial * angle.sin();
                    let z = length * 0.5;
                    let surface_local = x*cylinder.perp_x + y*cylinder.perp_y + z*cylinder.direction;
                    (surface_local, -random_dir)
                },
                2 => {  // Bottom cap 
                    let radial = rng.random_range(0.0..radius);
                    let angle = rng.random_range(0.0..std::f64::consts::TAU);
                    let x = radial * angle.cos();
                    let y = radial * angle.sin();
                    let z = -length * 0.5; 
                    let surface_local = x*cylinder.perp_x + y*cylinder.perp_y + z*cylinder.direction;
                    (surface_local, random_dir) 
                },
                _ => panic!("Invalid face selection"), // This should never happen
            };
            
            if cylinder.signed_distance(&surface_local).abs() > TOLERANCE {
                println!("{}", cylinder.signed_distance(&surface_local));
                panic!("Surface point not initialized correctly fix test")
            }

            let surface = wall_pos + surface_local;
    
            let result = cylinder.calculate_normal(&surface, &wall_pos, TOLERANCE);
    
            match result {
                Some(normal) => {
                    assert_approx_eq!(normal[0], expected_normal[0], 1e-12);
                    assert_approx_eq!(normal[1], expected_normal[1], 1e-12);
                    assert_approx_eq!(normal[2], expected_normal[2], 1e-12);
                }
                None => panic!("Normal calculation failed for cylinder"),
            }
        }
    }
}


