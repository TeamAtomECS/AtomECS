use bevy::prelude::*;
use crate::integrator::AtomECSBatchStrategy;
use crate::collisions::atom_collisions::CollisionParameters;
use crate::collisions::atom_collisions::ApplyCollisionsOption;
use crate::atom::{Position, Atom};
use nalgebra::Vector3;

/// Component that marks which box an atom is in for spatial partitioning
#[derive(Component)]
pub struct BoxID {
    /// ID of the box
    pub id: i64,
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