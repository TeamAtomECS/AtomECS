// An attempt at making wall_collision not so horrendously big

use nalgebra::Vector3;
use crate::shapes::{
    Cylinder,
    Cuboid,
    Sphere, 
    CylindricalPipe,
};

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

trait SDF {
    /// Calculate signed distance
    fn signed_distance(&self, point: &Vector3<f64>) -> f64;
}

/// Credit to Inigo Quilez's for the SDFs. Found at https://www.shadertoy.com/view/Xds3zN

/// Signed distance to a sphere.
impl SDF for Sphere{
    fn signed_distance(&self, point: &Vector3<f64>) -> f64 {
        point.norm() - &self.radius
    }
}

/// Signed distance to an axis-aligned box centered at the origin.
impl SDF for Cuboid{
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
impl SDF for Cylinder{
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
            // eprintln!("Speed 0 atom");
            return None;
        }

        let direction = -atom_vel / speed;
        let max_distance = speed * max_time;

        let mut distance_traveled = 0.0;

        for _ in 0..max_steps {
            let curr_pos = atom_pos + direction * distance_traveled;
            let local_pos = curr_pos - wall_pos;

            let distance_to_surface = self.signed_distance(&local_pos);

            if distance_to_surface < tolerance {
                if distance_traveled + distance_to_surface < max_distance + tolerance {
                    return Some(curr_pos);
                }
                else {
                    // eprintln!("Took too long to collide");
                    return None;
                }
            }

            distance_traveled += distance_to_surface;
        }
        // eprintln!("Too few steps for convergence. collision");
        None
    }
}

impl Intersect for CylindricalPipe {
    fn calculate_intersect(
        &self,
        atom_pos: &Vector3<f64>,
        atom_vel: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        max_time: f64,
        _tolerance: f64,
        _max_steps: i32,
    ) -> Option<Vector3<f64>> {
        let prev_pos = atom_pos - atom_vel * max_time;
        let delta_prev = prev_pos - wall_pos;
        let delta_curr = atom_pos - wall_pos;

        let axial_prev = delta_prev.dot(&self.direction);
        let radial_prev = delta_prev - axial_prev * self.direction;
        let axial_curr = delta_curr.dot(&self.direction);
        let radial_curr = delta_curr - axial_curr * self.direction;

        let a = (radial_curr - radial_prev).norm_squared();
        let b = 2.0 * radial_prev.dot(&(&radial_curr - &radial_prev));
        let c = radial_prev.norm_squared() - self.radius.powi(2);

        // Solve quadratic equation for t
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            // No intersection, should never happen
            return None;
        }
        let sqrt_discriminant = discriminant.sqrt();
        let t1 = (-b + sqrt_discriminant) / (2.0 * a);
        let t2 = (-b - sqrt_discriminant) / (2.0 * a);
        if (t1 <= 0.0 || t1 >= 1.0 ) && (t2 <= 0.0 || t2 >= 1.0 ) {
            return None;
        }

        let t = if t1 <= 0.0 || t1 >= 1.0  {
            t2
        } else if t2 <= 0.0 || t2 >= 1.0 {
            t1
        } else {
            f64::min(t1, t2)
        };

        // Calculate the collision point
        let collision_point = prev_pos * (1.0 - t) + atom_pos * t;
        if (collision_point - wall_pos).dot(&self.direction).abs() <= self.length * 0.5 {
            Some(collision_point)
        } else {
            None
        }
    }
}

impl Normal for Sphere {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>> {
        let normal_vector = point - wall_pos;

        // Gives the normal assuming collision from the inside
        if (normal_vector.norm() - self.radius).abs() < tolerance  {
            Some(-normal_vector.normalize())
        } else { None } // Should never happen
    }
}

impl Normal for Cuboid {
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

impl Normal for Cylinder {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>> {
        // Convert to local space
        let rel = point - wall_pos;
        let local = Vector3::new(
            rel.dot(&self.perp_x),
            rel.dot(&self.perp_y),
            rel.dot(&self.direction),
        );

        let axial_dist_to_surface = local.z.abs() - self.length * 0.5;
        let radial_dist_to_surface = (local.x.powi(2) + local.y.powi(2)).sqrt() - self.radius;
        // Gives the normal assuming collision from the inside
        if axial_dist_to_surface.abs() < tolerance && radial_dist_to_surface < 0.0 {
            let normal = self.direction * local.z.signum();
            Some(-normal) 
        } else if (radial_dist_to_surface).abs() < tolerance {
            let normal = self.perp_x * local.x + self.perp_y * local.y;
            Some(-normal.normalize()) 
        }
        else{
            None
        }
    }
}

impl Normal for CylindricalPipe {
    fn calculate_normal(&self, point: &Vector3<f64>, wall_pos: &Vector3<f64>, tolerance: f64) -> Option<Vector3<f64>> {
        // Convert to local space
        let rel = point - wall_pos;
        let local = Vector3::new(
            rel.dot(&self.perp_x),
            rel.dot(&self.perp_y),
            rel.dot(&self.direction),
        );

        let radial_dist_to_surface = (local.x.powi(2) + local.y.powi(2)).sqrt() - self.radius;
        if (radial_dist_to_surface).abs() < tolerance {
            let normal = self.perp_x * local.x + self.perp_y * local.y;
            Some(-normal.normalize()) 
        }
        else{
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Cylinder, Cuboid, Sphere, Volume};
    use crate::constant::PI;
    use rand::Rng;
    use assert_approx_eq::assert_approx_eq;


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

            let cuboid = Cuboid { half_width };
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
            let sphere = Sphere { radius };
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
            let random_dir = rand_vec(-1.0, 1.0);

            let wall_pos = rand_vec(-5.0, 5.0);
            let cylinder = Cylinder::new(radius, length, random_dir);

            let inside_local = {
                let radial_in = rng.random_range(0.0..radius);
                let angle_in = rng.random_range(0.0..std::f64::consts::TAU);
                let x_in = radial_in * angle_in.cos();
                let y_in = radial_in * angle_in.sin();
                let z_in = rng.random_range(-length * 0.5..length * 0.5);
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
            let sphere = Sphere { radius };
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

            let cuboid = Cuboid { half_width };
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
            let cylinder = Cylinder::new(radius, length, random_dir);
            let wall_pos = rand_vec(-5.0, 5.0);
    
            let face = rng.random_range(0..3);
    
            let (surface_local, expected_normal) = match face {
                0 => {  // Curved face
                    let angle = rng.random_range(0.0..2.0 * PI);
                    let x = radius * angle.cos();
                    let y = radius * angle.sin();
                    let z = rng.random_range(-length*0.5..length*0.5);
                    let surface_local = x * cylinder.perp_x + y * cylinder.perp_y + z * cylinder.direction;
                    let normal = -(x * cylinder.perp_x + y * cylinder.perp_y).normalize();
                    (surface_local, normal) 
                },
                1 => {  // Top cap
                    let radial = rng.random_range(0.0..radius);
                    let angle = rng.random_range(0.0..2.0 * PI);
                    let x = radial * angle.cos();
                    let y = radial * angle.sin();
                    let z = length * 0.5;
                    let surface_local = x*cylinder.perp_x + y*cylinder.perp_y + z*cylinder.direction;
                    (surface_local, -random_dir)
                },
                2 => {  // Bottom cap 
                    let radial = rng.random_range(0.0..radius);
                    let angle = rng.random_range(0.0..2.0 * PI);
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
}