use bevy::prelude::*;
use crate::integrator::AtomECSBatchStrategy;
use crate::collisions::atom_collisions::CollisionParameters;
use crate::collisions::atom_collisions::ApplyCollisionsOption;
use crate::collisions::wall_collisions::{Wall, MinWallDistance};
use crate::atom::{Position, Atom};
use crate::shapes::{Cylinder as MyCylinder, Cuboid as MyCuboid, Sphere as MySphere};
use nalgebra::Vector3;
use hashbrown::HashSet;
use crate::collisions::wall_collisions::NearWall;

/// Component that marks which box an atom is in for spatial partitioning
#[derive(Component)]
pub struct BoxID {
    /// ID of the box
    pub id: i64,
}

/// Resource that stores which boxes are close enough to walls to need collision checking
#[derive(Resource, Default)]
pub struct WallProximityMap {
    /// Set of box IDs that contain atoms that might be close to walls
    pub wall_affected_boxes: HashSet<i64>,
}

/// One-time calculation of which boxes are near walls (since walls are static)
pub fn calculate_wall_proximity_system(
    sphere_walls: Query<(&Position, &MySphere), With<Wall>>,
    cuboid_walls: Query<(&Position, &MyCuboid), With<Wall>>,
    cylinder_walls: Query<(&Position, &MyCylinder), With<Wall>>,
    params: Res<CollisionParameters>,
    min_distance: Res<MinWallDistance>,
    mut proximity_map: ResMut<WallProximityMap>,
    collision_option: Res<ApplyCollisionsOption>,
) {
    if !collision_option.apply_collision {
        return;
    }

    proximity_map.wall_affected_boxes.clear();
    
    let box_diagonal = params.box_width * 3.0_f64.sqrt();
    let threshold = min_distance.0 + box_diagonal / 2.0; 

    let half_grid = params.box_number / 2;

    let (start, end) = if params.box_number % 2 == 0 {
        (-half_grid, half_grid - 1)
    } else {
        (-half_grid, half_grid)
    };

    for x in start..=end {
        for y in start..=end {
            for z in start..=end {                
                let box_center = Vector3::new(
                x as f64 * params.box_width,
                y as f64 * params.box_width,
                z as f64 * params.box_width,
            );
            
            let box_id = pos_to_id(box_center, params.box_number, params.box_width);
                
                let mut is_near_wall = false;

                for (wall_pos, sphere) in sphere_walls.iter() {
                    let distance = sphere.is_near_wall(&box_center, &wall_pos.pos, min_distance.0);
                    if distance != 0 {
                        is_near_wall = true;
                        break;
                    }
                }
                
                if !is_near_wall {
                    for (wall_pos, cuboid) in cuboid_walls.iter() {
                        let distance = cuboid.is_near_wall(&box_center, &wall_pos.pos, min_distance.0);
                        if distance != 0 {
                            is_near_wall = true;
                            break;
                        }
                    }
                }
                
                if !is_near_wall {
                    for (wall_pos, cylinder) in cylinder_walls.iter() {
                        let distance = cylinder.is_near_wall(&box_center, &wall_pos.pos, min_distance.0);
                        if distance != 0 {
                            is_near_wall = true;
                            break;
                        }
                    }
                }
                
                if is_near_wall {
                    proximity_map.wall_affected_boxes.insert(box_id);
                }
            }
        }
    }
}

/// Initializes boxid component
pub fn init_boxid_system(
    mut query: Query<(Entity), (Without<BoxID>, With<Atom>)>,
    collisions_option: Res<ApplyCollisionsOption>,
    mut commands: Commands,
) {
    match collisions_option.apply_collision {
        false => return, // No need to initialize box IDs if collisions are not applied
        true => {
            query
                .iter_mut()
                .for_each(|(entity)| {
                    commands.entity(entity).insert(BoxID { id: 0 });
                });},
    }
}


/// Assigns box IDs to atoms
pub fn assign_boxid_system(
    mut query: Query<(&Position, &mut BoxID), With<Atom>>,
    collisions_option: Res<ApplyCollisionsOption>,
    params: Res<CollisionParameters>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    match collisions_option.apply_collision {
        false => return,
        true => {
            query
                .par_iter_mut()
                .batching_strategy(batch_strategy.0.clone())
                .for_each(|(position, mut boxid)| {
                    boxid.id = pos_to_id(position.pos, params.box_number, params.box_width);
                });
        }
    }
}

fn pos_to_id(pos: Vector3<f64>, n: i64, width: f64) -> i64 {
    //Assume that atoms that leave the grid are too sparse to collide, so disregard them
    //We'll assign them the max value of i64, and then check for this value when we do a collision and ignore them
    let bound = (n as f64) / 2.0 * width;

    let id: i64;
    if pos[0].abs() > bound || pos[1].abs() > bound || pos[2].abs() > bound {
        id = i64::MAX;
    } else {
        let xp: i64;
        let yp: i64;
        let zp: i64;

        // even number of boxes, vertex of a box is on origin
        // odd number of boxes, centre of a box is on the origin
        // grid cells run from [0, width), i.e include lower bound but exclude upper

        xp = (pos[0] / width + 0.5 * (n as f64)).floor() as i64;
        yp = (pos[1] / width + 0.5 * (n as f64)).floor() as i64;
        zp = (pos[2] / width + 0.5 * (n as f64)).floor() as i64;
        //convert position to box id
        id = xp + n * yp + n.pow(2) * zp;
    }

    id
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_pos_to_id() {
        let n: i64 = 10;
        let width: f64 = 2.0;

        let pos1 = Vector3::new(0.0, 0.0, 0.0);
        let pos2 = Vector3::new(1.0, 0.0, 0.0);
        let pos3 = Vector3::new(2.0, 0.0, 0.0);
        let pos4 = Vector3::new(9.9, 0.0, 0.0);
        let pos5 = Vector3::new(-9.9, 0.0, 0.0);
        let pos6 = Vector3::new(10.1, 0.0, 0.0);
        let pos7 = Vector3::new(-9.9, -9.9, -9.9);

        let id1 = pos_to_id(pos1, n, width);
        let id2 = pos_to_id(pos2, n, width);
        let id3 = pos_to_id(pos3, n, width);
        let id4 = pos_to_id(pos4, n, width);
        let id5 = pos_to_id(pos5, n, width);
        let id6 = pos_to_id(pos6, n, width);
        let id7 = pos_to_id(pos7, n, width);

        assert_eq!(id1, 555);
        assert_eq!(id2, 555);
        assert_eq!(id3, 556);
        assert_eq!(id4, 559);
        assert_eq!(id5, 550);
        assert_eq!(id6, i64::MAX);
        assert_eq!(id7, 0);
    }
}