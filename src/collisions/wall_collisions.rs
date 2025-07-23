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

pub trait NearWall{
    /// Check if an atom is near a wall
    fn is_near_wall(&self, 
        atom_pos: &Vector3<f64>, 
        wall_pos: &Vector3<f64>, 
        threshold: f64) 
        -> i32;
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
    fn calculate_normal(&self, point: Vector3<f64>) -> Vector3<f64>;
}

#[derive(Component)]
pub struct WallDistance {
    pub value: i32, // -1: inside/negative side, 0: far, 1: outside/positive side
}

/// Stores old wall distance value for collision detection
#[derive(Component)]
pub struct OldWallDistance {
    pub value: i32,
}

/// Minimum distance from a wall to consider an atom near it
#[derive(Resource)]
pub struct MinWallDistance(pub f64);

/// Check if an atom is near a sphere wall
impl NearWall for MySphere {
    fn is_near_wall(
        &self,
        atom_pos: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        threshold: f64,
    ) -> i32 {
        let distance_to_center = (atom_pos - wall_pos).norm();
        let distance_to_surface = distance_to_center - &self.radius;

        if distance_to_surface.abs() <= threshold {
            if distance_to_center >= self.radius { 1 } else { -1 }
        }
        else { 0 }
    }
}

/// Check if an atom is near a cuboid wall
impl NearWall for MyCuboid{
    fn is_near_wall(
        &self,
        atom_pos: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        threshold: f64,
    ) -> i32 {

        let local_pos = atom_pos - wall_pos;
        let mut is_near_wall = false;
        let mut is_outside = false;

        for axis in 0..3 {
            let coord_abs = local_pos[axis].abs();
            let extended_half_width = &self.half_width[axis] + threshold;
            
            if coord_abs >= extended_half_width {
                return 0;
            }
        }
        
        for axis in 0..3 {
            let dist_pos = local_pos[axis] - &self.half_width[axis];
            let dist_neg = local_pos[axis] + &self.half_width[axis];
            
            if dist_pos.abs() < threshold {
                is_near_wall = true;
                if dist_pos >= 0.0 {
                    is_outside = true; 
                }
            }
            else if dist_neg.abs() < threshold {
                is_near_wall = true;
                if dist_neg <= 0.0 {
                    is_outside = true; 
                }
            }
        }
        
        if is_near_wall {
            if is_outside { 1 } else { -1 }
        } else {
            0
        }
    }
}

/// Check if an atom is near a cylinder wall
impl NearWall for MyCylinder {
    fn is_near_wall(
        &self,
        atom_pos: &Vector3<f64>,
        wall_pos: &Vector3<f64>,
        threshold: f64,
    ) -> i32 {
        let delta = atom_pos - wall_pos;
        let axial_projection = delta.dot(&self.direction);
        
        // atom must be within extended bounds of cylinder
        let extended_half_length = &self.length / 2.0 + threshold;
        let extended_radius = &self.radius + threshold;
        
        if axial_projection.abs() > extended_half_length {
            return 0; 
        }
        
        let radial_component = delta - (axial_projection * &self.direction);
        let radial_distance = radial_component.norm();
        
        if radial_distance > extended_radius {
            return 0;
        }
        
        let half_length = &self.length / 2.0;
        let axial_dist_to_cap = axial_projection.abs() - half_length;
        let radial_dist_to_surface = radial_distance - &self.radius;
        
        let mut is_near_wall = false;
        let mut is_outside = false;
        
        if radial_dist_to_surface.abs() < threshold {
            is_near_wall = true;
            if radial_dist_to_surface >= 0.0 {
                is_outside = true; 
            }
        }
        
        if axial_dist_to_cap.abs() < threshold {
            is_near_wall = true;
            if axial_dist_to_cap >= 0.0 {
                is_outside = true; 
            }
        }
        
        if is_near_wall {
            if is_outside { 1 } else { -1 }
        } else {
            0
        }
    }
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

/// Initialize all atoms without WallDistance component with default Far value
fn init_wall_distance_system(
    mut commands: Commands,
    query: Query<Entity, (With<Atom>, Without<WallDistance>)>,
) {
    // Inits Old WallDistance as well. Presumes that all atoms wuthout WallDistance also lack OldWallDistance.
    for atom_entity in query.iter() {
        commands.entity(atom_entity).insert(WallDistance { value: 0 });
        commands.entity(atom_entity).insert(OldWallDistance { value: 0 }); 
    }
}

/// System that updates WallDistance for all atoms based on proximity to walls
fn update_wall_distances_system(
    mut atom_query: Query<(&Position, &mut WallDistance, &mut OldWallDistance), With<Atom>>,
    sphere_walls: Query<(&Position, &MySphere), With<Wall>>,
    cuboid_walls: Query<(&Position, &MyCuboid), With<Wall>>,
    cylinder_walls: Query<(&Position, &MyCylinder), With<Wall>>,
    min_distance: Res<MinWallDistance>,
    batch_strategy: Res<AtomECSBatchStrategy>, 
) {
    // Currently not sure how to handle walls close to each other. Just estanblishes an order for now.
    
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(atom_pos, mut wall_distance, mut old_wall_distance)| {
            // Reset old distance to current value
            old_wall_distance.value = wall_distance.value;
            let mut new_distance = 0;
            
            for (wall_pos, sphere) in sphere_walls.iter() {
                let distance = sphere.is_near_wall(&atom_pos.pos, &wall_pos.pos, min_distance.0);
                if distance != 0 {
                    new_distance = distance;
                    break;
                }
            }
            
            if new_distance == 0 {
                for (wall_pos, cuboid) in cuboid_walls.iter() {
                    let distance = cuboid.is_near_wall(&atom_pos.pos, &wall_pos.pos, min_distance.0);
                    if distance != 0 {
                        new_distance = distance;
                        break;
                    }
                }
            }
            
            if new_distance == 0 {
                for (wall_pos, cylinder) in cylinder_walls.iter() {
                    let distance = cylinder.is_near_wall(&atom_pos.pos, &wall_pos.pos, min_distance.0);
                    if distance != 0 {
                        new_distance = distance;
                        break;
                    }
                }
            }
            
            wall_distance.value = new_distance;
        });
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, World};
    use crate::shapes::Sphere;
    use crate::atom::{Atom, Position};
    use nalgebra::Vector3;

    #[test]
    fn test_sphere_near_wall_detection() {
        let threshold = 0.2;
        
        // Create sphere wall
        let sphere = Sphere { radius: 5.0 };
        let wall_pos = Vector3::new(0.0, 0.0, 0.0);
        let epsilon = 1e-15; 

        let test_cases = vec![
            (Vector3::new(4.85, 0.0, 0.0), -1, "Inside sphere, near wall"),
            (Vector3::new(0.0, 4.9, 0.0), -1, "Inside sphere, near wall Y-axis"),
            (Vector3::new(0.0, 0.0, -4.85), -1, "Inside sphere, near wall -Z axis"),
            (Vector3::new(5.15, 0.0, 0.0), 1, "Outside sphere, near wall"),
            (Vector3::new(3.0, 4.0, 0.0), 1, "Exactly on sphere surface"),
            (Vector3::new(1.0, 1.0, 1.0), 0, "Inside sphere, far from wall"),
            (Vector3::new(8.0, 0.0, 0.0), 0, "Outside sphere, far from wall"),
            (Vector3::new(0.0, 1e10, 0.0), 0, "Outside sphere, very far"),
            (Vector3::new(4.8 - epsilon, 0.0, 0.0), 0, "Inside, just beyond threshold (dist_to_surface = -0.21)"),
            (Vector3::new(5.2 + epsilon, 0.0, 0.0), 0, "Outside, just beyond threshold (dist_to_surface = 0.21)"),
        ];

        for (atom_pos, expected, description) in test_cases {
            let result = sphere.is_near_wall(&atom_pos, &wall_pos, threshold);
            assert_eq!(result, expected, "Failed for case: {}", description);
        }
    }

    #[test]
    fn test_cuboid_near_wall_detection() {
        let threshold = 0.2;
        
        let cuboid = MyCuboid { half_width: Vector3::new(1.0, 2.0, 3.0) };
        let wall_pos = Vector3::new(0.0, 0.0, 0.0);
        let epsilon = 1e-15;
        
        let test_cases = vec![
            (Vector3::new(0.85, 0.0, 0.0), -1, "Inside cuboid, near X wall"),
            (Vector3::new(0.0, 1.9, 0.0), -1, "Inside cuboid, near Y wall"),
            (Vector3::new(0.0, 0.0, -2.85), -1, "Inside cuboid, near -Z wall"),
            (Vector3::new(1.15, 0.0, 0.0), 1, "Outside cuboid, near +X wall"),
            (Vector3::new(0.0, -2.1, 0.0), 1, "Outside cuboid, near -Y wall"),
            (Vector3::new(0.0, 0.0, 3.19), 1, "Outside cuboid, near +Z wall"),
            (Vector3::new(5.0, 2.0, 3.0), 0, "Outside cuboid, out of bounds"),
            (Vector3::new(0.8 - epsilon, 0.0, 0.0), 0, "Inside, just beyond threshold"),
            (Vector3::new(1.2 + epsilon, 0.0, 0.0), 0, "Outside, just beyond threshold"),
        ];

        for (atom_pos, expected, description) in test_cases {
            let result = cuboid.is_near_wall(&atom_pos, &wall_pos, threshold);
            assert_eq!(result, expected, "Failed for case: {}", description);
        }
    }

    #[test]
    fn test_cylinder_near_wall_detection() {
        let threshold = 0.2;
        let epsilon = 1e-15; 
        
        let cylinder = MyCylinder::new(2.0, 4.0, Vector3::new(0.0, 0.0, 1.0));
        let wall_pos = Vector3::new(0.0, 0.0, 0.0);

        let test_cases = vec![
            (Vector3::new(1.85, 0.0, 0.0), -1, "Inside cylinder, near radial wall"),
            (Vector3::new(0.0, 1.9, 0.0), -1, "Inside cylinder, near radial wall"),
            (Vector3::new(2.15, 0.0, 0.0), 1, "Outside cylinder, near radial wall"),
            (Vector3::new(0.0, -2.1, 0.0), 1, "Outside cylinder, near radial wall"),
            (Vector3::new(0.0, 0.0, 1.85), -1, "Inside cylinder, near +Z cap"),
            (Vector3::new(0.0, 0.0, 2.15), 1, "Outside cylinder, near +Z cap"),
            (Vector3::new(0.0, 0.0, -2.1), 1, "Outside cylinder, near -Z cap "),
            (Vector3::new(1.0, 0.0, 0.0), 0, "Inside cylinder, far"),
            (Vector3::new(1.8 - epsilon, 0.0, 0.0), 0, "Just beyond radial threshold, Inside"),
            (Vector3::new(2.2 + epsilon, 0.0, 0.0), 0, "Just beyond radial threshold, Outside"),
            (Vector3::new(0.0, 0.0, 1.8 - epsilon), 0, "Just beyond axial threshold, Inside"),
            (Vector3::new(0.0, 0.0, 2.2 + epsilon), 0, "Just beyond axial threshold, Outside"),
        ];

        for (atom_pos, expected, description) in test_cases {
            let result = cylinder.is_near_wall(&atom_pos, &wall_pos, threshold);
            assert_eq!(result, expected, "Failed for case: {}", description);
        }
    }
}