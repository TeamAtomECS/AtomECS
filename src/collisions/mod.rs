pub mod atom_collisions;
pub mod wall_collisions;
pub mod spatial_grid;

use bevy::prelude::*;
use crate::collisions::atom_collisions::*;
use crate::integrator::IntegrationSet;
use crate::collisions::wall_collisions::*;
use crate::collisions::spatial_grid::*;

pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        // Note that the collisions system must be applied after the velocity integrator or it will violate conservation of energy and cause heating
        app.add_systems(PostUpdate, init_boxid_system.after(IntegrationSet::EndIntegration));
        app.add_systems(PostUpdate, assign_boxid_system.after(init_boxid_system));
        app.add_systems(PostUpdate, apply_collisions_system.after(assign_boxid_system));
    }
}