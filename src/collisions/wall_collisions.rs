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
pub struct MaxSteps(pub i32); // Max steps for sdf convergence

#[derive(Resource)]
pub struct SurfaceThreshold(pub f64); // Minimum penetration for detecting a collision

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
            eprintln!("Speed 0 atom");
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
                if distance_traveled + distance_to_surface < max_distance {
                    return Some(current_pos);
                }
                else {
                    eprintln!("Took too long to collide");
                    return None;
                }
            }

            distance_traveled += distance_to_surface;
        }
        eprintln!("Too few steps for convergence. collision");
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
        eprintln!("Distance to travel set to a negative value somehow");
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
    wall_query: Query<(Entity, &T, &Wall, &Position), Without<Atom>>,
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
    use crate::atom::{Atom, Position, Velocity};
    use bevy::prelude::*;
    use crate::shapes::{
        Cylinder as MyCylinder,
        Cuboid as MyCuboid,
        Sphere as MySphere,
        Volume
    };


    const TEST_RUNS: usize = 10_000;
    const TOLERANCE: f64 = 1e-9;
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

            let velocity = rand_vec(-1.0, 1.0).normalize();
            let outside_local = inside_local + velocity*10.0;

            let inside = wall_pos + inside_local;
            let outside = wall_pos + outside_local;

            if cuboid.contains(&wall_pos, &outside) || !cuboid.contains(&wall_pos, &inside) {
                panic!("Wrong initialization in test");
            }

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

            let velocity = rand_vec(-1.0, 1.0).normalize();
            let outside_local = inside_local + velocity*radius*2.0;

            let inside = wall_pos + inside_local;
            let outside = wall_pos + outside_local;

            if sphere.contains(&wall_pos, &outside) || !sphere.contains(&wall_pos, &inside) {
                panic!("Wrong initialization in test");
            }

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

            let inside_local = {
                let radial_in = rng.random_range(0.0..radius);
                let angle_in = rng.random_range(0.0..std::f64::consts::TAU);
                let x_in = radial_in * angle_in.cos();
                let y_in = radial_in * angle_in.sin();
                let z_in = rng.random_range(-length * 0.4..length * 0.4);
                let inside = cylinder.perp_x * x_in + cylinder.perp_y * y_in + cylinder.direction * z_in;
                inside
            };

            let velocity = rand_vec(-1.0, 1.0).normalize();
            let outside_local = inside_local + velocity*22.5;

            let inside = wall_pos + inside_local;
            let outside = wall_pos + outside_local;

            if cylinder.contains(&wall_pos, &outside) || !cylinder.contains(&wall_pos, &inside) {
                panic!("Wrong initialization in test");
            }

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
        
            let face = rng.random_range(0..6); 
        
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
    
    // This is more complicated than the thing its testing
    #[test]
    fn test_normal_cylinder() {
        for _ in 0..TEST_RUNS {
            let mut rng = rand::rng();
            let radius = 1.0;
            let length = 2.0;
            let random_dir = Vector3::new(0.0,0.0,1.0); // Random direction
            let cylinder = MyCylinder::new(radius, length, random_dir);
            let wall_pos = rand_vec(-5.0, 5.0);
    
            let face = rng.random_range(0..3);
    
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
                _ => panic!("Invalid face selection"),
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

    // this is ugly as sin
    #[test]
    fn test_collision_check() {
        let mut app = App::new();
        let position1 = Position {pos: Vector3::new(5.0, 0.0, 0.0)};
        let velocity1= Velocity {vel: Vector3::new(1.0, 0.0, 0.0)};
        let dt = 1.0 + 1e-15; // seems a bit too imprecise? maybe
        let atom1 = app.world_mut().spawn(position1.clone()).insert(velocity1.clone()).insert(Atom).id();

        let wall_pos1 = Position {pos: Vector3::new(0.0, 0.0, 0.0)};
        let sphere = MySphere{radius: 4.0 };
        let wall_type = Wall {volume_type: VolumeType::Inclusive};
        let sphere_wall = app.world_mut()
            .spawn(wall_pos1.clone())
            .insert(Wall {volume_type: VolumeType::Inclusive})
            .insert(MySphere{radius: 4.0 })
            .id();

        let result1 = collision_check((atom1, &position1, &velocity1), (sphere_wall, &sphere, &wall_type, &wall_pos1), dt, TOLERANCE, MAX_STEPS);
        match result1 {
            Some(collision_info) => {
                assert_eq!(atom1, collision_info.atom);
                assert_eq!(sphere_wall, collision_info.wall);
                assert_approx_eq!(4.0, collision_info.collision_point[0], TOLERANCE);
                assert_approx_eq!(0.0, collision_info.collision_point[1], TOLERANCE);
                assert_approx_eq!(0.0, collision_info.collision_point[2], TOLERANCE);
                assert_approx_eq!(-1.0, collision_info.collision_normal[0], 1e-14);
                assert_approx_eq!(0.0, collision_info.collision_normal[1], 1e-14);
                assert_approx_eq!(0.0, collision_info.collision_normal[2], 1e-14);
            }
            None => {
                panic!();
            }
        }

        app.world_mut().despawn(atom1);
        app.world_mut().despawn(sphere_wall);


        let position2 = Position {pos: Vector3::new(5.0, 0.0, 0.0)};
        let velocity2 = Velocity {vel: Vector3::new(1.0, 0.0, 0.0)};
        let dt = 1.0 + 1e-15; // seems a bit too imprecise? maybe
        let atom2 = app.world_mut().spawn(position2.clone()).insert(velocity2.clone()).insert(Atom).id();

        let wall_pos2 = Position {pos: Vector3::new(0.0, 0.0, 0.0)};
        let cuboid = MyCuboid{half_width: Vector3::new(4.0, 1.0, 1.0)};
        let wall_type = Wall {volume_type: VolumeType::Inclusive};
        let cuboid_wall = app.world_mut()
            .spawn(wall_pos2.clone())
            .insert(Wall {volume_type: VolumeType::Inclusive})
            .insert(MyCuboid{half_width: Vector3::new(4.0, 1.0, 1.0)})
            .id();

        let result2 = collision_check((atom2, &position2, &velocity2), (cuboid_wall, &cuboid, &wall_type, &wall_pos1), dt, TOLERANCE, MAX_STEPS);
        match result2 {
            Some(collision_info) => {
                assert_eq!(atom2, collision_info.atom);
                assert_eq!(cuboid_wall, collision_info.wall);
                assert_approx_eq!(4.0, collision_info.collision_point[0], TOLERANCE);
                assert_approx_eq!(0.0, collision_info.collision_point[1], TOLERANCE);
                assert_approx_eq!(0.0, collision_info.collision_point[2], TOLERANCE);
                assert_approx_eq!(-1.0, collision_info.collision_normal[0], 1e-14);
                assert_approx_eq!(0.0, collision_info.collision_normal[1], 1e-14);
                assert_approx_eq!(0.0, collision_info.collision_normal[2], 1e-14);
            }
            None => {
                panic!();
            }
        }

        app.world_mut().despawn(atom2);
        app.world_mut().despawn(cuboid_wall);

        let position3 = Position {pos: Vector3::new(5.0, 0.0, 0.0)};
        let velocity3 = Velocity {vel: Vector3::new(1.0, 0.0, 0.0)};
        let dt = 1.0 + 1e-15;
        let atom3 = app.world_mut().spawn(position3.clone()).insert(velocity3.clone()).insert(Atom).id();

        let wall_pos3 = Position {pos: Vector3::new(0.0, 0.0, 0.0)};
        let cylinder = MyCylinder::new(1.0, 8.0, Vector3::new(1.0, 0.0, 0.0));
        let wall_type = Wall {volume_type: VolumeType::Inclusive};
        let cylinder_wall = app.world_mut()
            .spawn(wall_pos3.clone())
            .insert(Wall {volume_type: VolumeType::Inclusive})
            .insert(MyCylinder::new(1.0, 8.0, Vector3::new(1.0, 0.0, 0.0)))
            .id();

        let result3 = collision_check((atom3, &position3, &velocity3), (cylinder_wall, &cylinder, &wall_type, &wall_pos1), dt, TOLERANCE, MAX_STEPS);
        match result3 {
            Some(collision_info) => {
                assert_eq!(atom3, collision_info.atom);
                assert_eq!(cylinder_wall, collision_info.wall);
                assert_approx_eq!(4.0, collision_info.collision_point[0], TOLERANCE);
                assert_approx_eq!(0.0, collision_info.collision_point[1], TOLERANCE);
                assert_approx_eq!(0.0, collision_info.collision_point[2], TOLERANCE);
                assert_approx_eq!(-1.0, collision_info.collision_normal[0], 1e-14);
                assert_approx_eq!(0.0, collision_info.collision_normal[1], 1e-14);
                assert_approx_eq!(0.0, collision_info.collision_normal[2], 1e-14);
            }
            None => {
                panic!();
            }
        }
    }

    // so is this
    #[test]
    fn test_do_wall_collision() {
        let mut app = App::new();
        let dt = 1.0;
        let position = Position {pos: Vector3::new(1.0, 1.0, 0.0)};
        let velocity = Velocity {vel: Vector3::new(2.0,2.0,0.0)};
        let length = velocity.vel.norm()*dt;
        let distance = DistanceToTravel{distance_to_travel: length};
        let atom = app.world_mut()
            .spawn(Atom)
            .insert(position.clone())
            .insert(velocity.clone())
            .insert(DistanceToTravel{distance_to_travel: length})
            .id();
        let wall = app.world_mut().spawn(Wall { volume_type: VolumeType::Inclusive}).id();
        let collision_point = Vector3::new(0.0,0.0,0.0);
        let collision_normal = Vector3::new(-1.0,0.0,0.0);
        let collision_info = CollisionInfo {
            atom,
            wall,
            collision_point,
            collision_normal,
        };

        fn test_system(
            mut query: Query<(Entity, &mut Position, &mut Velocity, &mut DistanceToTravel)>,
            wall: Query<Entity, With<Wall>>) {
            query.iter_mut().for_each(|(atom, mut atom_pos, mut atom_vel, mut distance)| {
                for wall in wall.iter() {
                    let dt = 1.0; // If you change this make sure to change the dt above as well
                    let collision_point = Vector3::new(0.0,0.0,0.0);
                    let collision_normal = Vector3::new(-1.0,0.0,0.0);
                    let collision_info = CollisionInfo {
                        atom,
                        wall,
                        collision_point,
                        collision_normal,
                    };
                    let time_used = do_wall_collision((&mut atom_pos, &mut atom_vel, &mut distance), &collision_info, dt);
                    assert_approx_eq!(time_used, ((atom_pos.pos - atom_vel.vel*dt) - collision_point).norm()/atom_vel.vel.norm())
                }
            });
        }
        app.add_systems(Update, test_system);
        app.update();

        let new_velocity = app.world()
            .entity(atom)
            .get::<Velocity>()
            .expect("Entity not found!")
            .vel;
        let new_position = app.world()
            .entity(atom)
            .get::<Position>()
            .expect("Entity not found")
            .pos;
        let new_distance = app.world()
            .entity(atom)
            .get::<DistanceToTravel>()
            .expect("Entity not found")
            .distance_to_travel;
            
        let time_used = ((position.pos - velocity.vel*dt) - collision_point).norm()/velocity.vel.norm();
        let distance_travelled = ((position.pos - velocity.vel*dt) - collision_point).norm();

        assert_ne!(new_velocity, velocity.vel);
        assert_approx_eq!(new_position[0], collision_point[0] + new_velocity[0]*(dt - time_used), 1e-15);
        assert_approx_eq!(new_position[1], collision_point[1] + new_velocity[1]*(dt - time_used), 1e-15);
        assert_approx_eq!(new_position[2], collision_point[2] + new_velocity[2]*(dt - time_used), 1e-15);
        assert_approx_eq!(distance.distance_to_travel - distance_travelled, new_distance, 1e-15);
    }

    //// Test multiple collisions
    //// Basically an integration test
    //// Needs to be changed to work with any reflection. 
    //// Will also fail as is because OldForces Force element is private
    //// But with only specular reflection it does work 
    // #[test]
    // fn test_systems() {
    //     use crate::integrator::{IntegrationPlugin, IntegrationSet, Timestep, AtomECSBatchStrategy, OldForce};
    //     use crate::initiate::NewlyCreated;
    //     use crate::atom::{Force, Mass};

    //     let mut app = App::new();
    //     app.world_mut()
    //         .spawn(Wall{volume_type: VolumeType::Inclusive})
    //         .insert(MyCuboid{half_width: Vector3::new(1.0, 1.0, 1.0)})
    //         .insert(Position{pos: Vector3::new(0.0, 0.0, 0.0)});
    //     let atom = app.world_mut()
    //         .spawn(Atom)
    //         .insert(Position{pos: Vector3::new(-1.0, 0.0, 0.0)})
    //         .insert(Velocity{vel: Vector3::new(5.0, -5.0, 0.0)})
    //         .insert(NewlyCreated)
    //         .insert(Force::default())
    //         .insert(Mass { value: 1.0 })
    //         .insert(OldForce(Force::default()))
    //         .id();
    //     app.add_plugins(IntegrationPlugin);
    //     app.insert_resource(Timestep {delta: 1.0});
    //     app.insert_resource(MaxSteps(1000));
    //     app.insert_resource(SurfaceThreshold(1e-9));
    //     app.insert_resource(AtomECSBatchStrategy::default());
    //     app.add_systems(PreUpdate, init_distance_to_travel_system.before(IntegrationSet::BeginIntegration));
    //     app.add_systems(Update, wall_collision_system::<MyCuboid>);
    //     app.add_systems(PostUpdate, reset_distance_to_travel_system);
    //     app.update();

    //     let new_position = app.world()
    //         .entity(atom)
    //         .get::<Position>()
    //         .expect("Entity not found")
    //         .pos;

    //     println!("{}", new_position);
    //     assert_approx_eq!(new_position[0], 0.0, 10*TOLERANCE);
    //     assert_approx_eq!(new_position[1], -1.0, 10*TOLERANCE);
    //     assert_approx_eq!(new_position[2], 0.0, 10*TOLERANCE);
    }
}


