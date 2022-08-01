use bevy::{prelude::*};

use crate::atom::Position;

pub struct Scale(pub f64);

// Copy atom positions into entity transform positions for rendering purposes.
pub fn copy_positions(
    mut query: Query<(&Position, &mut Transform)>,
    scale: Res<Scale>
) {
    query.par_for_each_mut(512, |(pos, mut transform)|
    {
        transform.translation = Vec3::new((scale.0 * pos.pos[0]) as f32, (scale.0 * pos.pos[1]) as f32, (scale.0 * pos.pos[2]) as f32);
    }
    );
}