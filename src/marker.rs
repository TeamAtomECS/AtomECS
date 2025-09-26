use bevy::prelude::*;
use crate::atom::{Atom, Position, Velocity};
use crate::collisions::CollisionsSet;

#[derive(Component)]
pub struct Marker;

pub fn add_marker_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &Velocity), With<Atom>>,
) {
    for (entity, pos, vel) in query.iter() {
        // if pos.pos.x < 210e-6 && pos.pos.x > 200e-6 {
        //     commands.entity(entity).insert(Marker);
        // }
        // else if pos.pos.x > 210e-6 {
        //     commands.entity(entity).remove::<Marker>();
        // }
        if pos.pos.x > 0.0 {
            commands.entity(entity).insert(Marker);
        }
    }
}

pub struct MarkerPlugin;
impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, add_marker_system.after(CollisionsSet::WallCollisionSystems));
    }
}