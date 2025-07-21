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

#[derive(Component)]
pub struct WallDistance {
    pub value: i32, // -1: inside/negative side, 0: far, 1: outside/positive side
}

/// Minimum distance from a wall to consider an atom "near" it
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

        // First check: atom must be within threshold distance of the cuboid bounds
        for axis in 0..3 {
            let coord_abs = local_pos[axis].abs();
            let extended_half_width = &self.half_width[axis] + threshold;
            
            if coord_abs >= extended_half_width {
                return 0; // Too far from cuboid entirely
            }
        }
        
        // Now apply your existing logic for atoms that are actually near the cuboid
        for axis in 0..3 {
            let dist_pos = local_pos[axis] - &self.half_width[axis];
            let dist_neg = local_pos[axis] + &self.half_width[axis];
            
            if dist_pos.abs() < threshold {
                is_near_wall = true;
                if dist_pos >= 0.0 {
                    is_outside = true; // Outside the cuboid
                }
            }
            else if dist_neg.abs() < threshold {
                is_near_wall = true;
                if dist_neg <= 0.0 {
                    is_outside = true; // Outside the cuboid
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
        
        // First check: atom must be within extended bounds of cylinder
        let extended_half_length = &self.length / 2.0 + threshold;
        let extended_radius = &self.radius + threshold;
        
        if axial_projection.abs() > extended_half_length {
            return 0; // Too far from cylinder axially
        }
        
        // Calculate radial distance from cylinder axis
        let radial_component = delta - (axial_projection * &self.direction);
        let radial_distance = radial_component.norm();
        
        if radial_distance > extended_radius {
            return 0; // Too far from cylinder radially
        }
        
        // Now check what type of wall we're near
        let half_length = &self.length / 2.0;
        let axial_dist_to_cap = axial_projection.abs() - half_length;
        let radial_dist_to_surface = radial_distance - &self.radius;
        
        let mut is_near_wall = false;
        let mut is_outside = false;
        
        // Check if near cylindrical surface
        if radial_dist_to_surface.abs() < threshold {
            is_near_wall = true;
            if radial_dist_to_surface >= 0.0 {
                is_outside = true; // Outside the cylindrical surface
            }
        }
        
        // Check if near end caps (only if within radial bounds)
        if axial_dist_to_cap.abs() < threshold {
            is_near_wall = true;
            if axial_dist_to_cap >= 0.0 {
                is_outside = true; // Outside the end caps
            }
        }
        
        if is_near_wall {
            if is_outside { 1 } else { -1 }
        } else {
            0
        }
    }
}

/// Initialize all atoms without WallDistance component with default Far value
fn init_wall_distance_system(
    mut commands: Commands,
    query: Query<Entity, (With<Atom>, Without<WallDistance>)>,
) {
    for atom_entity in query.iter() {
        commands.entity(atom_entity).insert(WallDistance { value: 0 }); 
    }
}

/// System that updates WallDistance for all atoms based on proximity to walls
fn update_wall_distances_system(
    mut atom_query: Query<(&Position, &mut WallDistance), With<Atom>>,
    sphere_walls: Query<(&Position, &MySphere), With<Wall>>,
    cuboid_walls: Query<(&Position, &MyCuboid), With<Wall>>,
    cylinder_walls: Query<(&Position, &MyCylinder), With<Wall>>,
    min_distance: Res<MinWallDistance>,
    batch_strategy: Res<AtomECSBatchStrategy>, 
) {
    // use rayon::prelude::*;
    // Currently not sure how to handle walls close to each other. This just extablishes a precedence for now.
    
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(atom_pos, mut wall_distance)| {
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

    // Helper to create test app with resources
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(MinWallDistance(0.2)); // 0.2 threshold
        app
    }

    #[test]
    fn test_sphere_near_wall_detection() {
        let threshold = 0.2;
        
        // Create sphere wall
        let sphere = Sphere { radius: 5.0 };
        let wall_pos = Vector3::new(0.0, 0.0, 0.0);
        let epsilon = 1e-15; // For numerical precision testing

        // Test cases: (atom_pos, expected_result, description)
        let test_cases = vec![
            (Vector3::new(4.85, 0.0, 0.0), -1, "Inside sphere, near wall (dist_to_surface = -0.15)"),
            (Vector3::new(0.0, 4.9, 0.0), -1, "Inside sphere, near wall Y-axis (dist_to_surface = -0.1)"),
            (Vector3::new(0.0, 0.0, -4.85), -1, "Inside sphere, near wall -Z axis (dist_to_surface = -0.15)"),
            (Vector3::new(5.15, 0.0, 0.0), 1, "Outside sphere, near wall (dist_to_surface = 0.15)"),
            (Vector3::new(0.0, -5.1, 0.0), 1, "Outside sphere, near wall -Y axis (dist_to_surface = 0.1)"),
            (Vector3::new(5.0, 0.0, 0.0), 1, "Exactly on sphere surface +X"),
            (Vector3::new(0.0, -5.0, 0.0), 1, "Exactly on sphere surface -Y"),
            (Vector3::new(3.0, 4.0, 0.0), 1, "Exactly on sphere surface (3,4,0) - distance = 5.0"),
            (Vector3::new(1.0, 1.0, 1.0), 0, "Inside sphere, far from wall (dist_to_surface ≈ -3.27)"),
            (Vector3::new(8.0, 0.0, 0.0), 0, "Outside sphere, far from wall (dist_to_surface = 3.0)"),
            (Vector3::new(0.0, 10.0, 0.0), 0, "Outside sphere, very far (dist_to_surface = 5.0)"),
            (Vector3::new(6.0, 6.0, 6.0), 0, "Outside sphere, far diagonal (dist_to_surface ≈ 5.39)"),
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
        
        // Create cuboid wall (2x4x6 box)
        let cuboid = MyCuboid { half_width: Vector3::new(1.0, 2.0, 3.0) };
        let wall_pos = Vector3::new(0.0, 0.0, 0.0);
        let epsilon = 1e-15;
        
        // Test cases: (atom_pos, expected_result, description)
        let test_cases = vec![
            (Vector3::new(0.85, 0.0, 0.0), -1, "Inside cuboid, near X wall (dist = -0.15)"),
            (Vector3::new(0.0, 1.9, 0.0), -1, "Inside cuboid, near Y wall (dist = -0.1)"),
            (Vector3::new(0.0, 0.0, -2.85), -1, "Inside cuboid, near -Z wall (dist = -0.15)"),
            (Vector3::new(1.15, 0.0, 0.0), 1, "Outside cuboid, near +X wall (dist = 0.15)"),
            (Vector3::new(0.0, -2.1, 0.0), 1, "Outside cuboid, near -Y wall (dist = 0.1)"),
            (Vector3::new(0.0, 0.0, 3.19), 1, "Outside cuboid, near +Z wall (dist = 0.19)"),
            (Vector3::new(-1.0, 0.0, 0.0), 1, "Exactly on cuboid surface +X"),
            (Vector3::new(0.0, -2.0, 0.0), 1, "Exactly on cuboid surface -Y"),
            (Vector3::new(0.0, 0.0, 3.0), 1, "Exactly on cuboid surface +Z"),
            (Vector3::new(5.0, 2.0, 3.0), 0, "Outside cuboid, out of bounds"),
            (Vector3::new(1.0, 2.0, 3.0), 1, "At cuboid vertex"),
            (Vector3::new(0.2, 2.0, 1.0), 1, "Inside cuboid, far from all walls"),
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
        let epsilon = 1e-15; // For numerical precision testing
        
        // Create cylinder wall (radius=2, length=4, along Z-axis)
        let cylinder = MyCylinder::new(2.0, 4.0, Vector3::new(0.0, 0.0, 1.0));
        let wall_pos = Vector3::new(0.0, 0.0, 0.0);
        
        // Test cases: (atom_pos, expected_result, description)
        let test_cases = vec![
            (Vector3::new(1.85, 0.0, 0.0), -1, "Inside cylinder, near radial wall (dist = -0.15)"),
            (Vector3::new(0.0, 1.9, 0.0), -1, "Inside cylinder, near radial wall Y (dist = -0.1)"),
            (Vector3::new(2.15, 0.0, 0.0), 1, "Outside cylinder, near radial wall (dist = 0.15)"),
            (Vector3::new(0.0, -2.1, 0.0), 1, "Outside cylinder, near radial wall -Y (dist = 0.1)"),
            (Vector3::new(0.0, 0.0, 1.85), -1, "Inside cylinder, near +Z cap (dist = -0.15)"),
            (Vector3::new(0.0, 0.0, -1.9), -1, "Inside cylinder, near -Z cap (dist = -0.1)"),
            (Vector3::new(0.0, 0.0, 2.15), 1, "Outside cylinder, near +Z cap (dist = 0.15)"),
            (Vector3::new(0.0, 0.0, -2.1), 1, "Outside cylinder, near -Z cap (dist = 0.1)"),
            (Vector3::new(2.0, 0.0, 0.0), 1, "Exactly on cylinder surface radial"),
            (Vector3::new(0.0, 0.0, 2.0), 1, "Exactly on cylinder end cap"),
            (Vector3::new(1.0, 0.0, 0.0), 0, "Inside cylinder, far from radial wall"),
            (Vector3::new(0.5, 0.5, 0.5), 0, "Inside cylinder, far from all walls"),
            (Vector3::new(5.0, 0.0, 0.0), 0, "Outside cylinder, far radially"),
            (Vector3::new(0.0, 0.0, 5.0), 0, "Outside cylinder, far axially"),
            (Vector3::new(3.0, 3.0, 3.0), 0, "Outside cylinder, far diagonal"),
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