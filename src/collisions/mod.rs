pub mod atom_collisions;
pub mod wall_collisions;

use bevy::prelude::*;
use crate::collisions::atom_collisions::{apply_collisions_system, init_boxid_system,};
use crate::integrator::IntegrationSet;

pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        // Note that the collisions system must be applied after the velocity integrator or it will violate conservation of energy and cause heating
        app.add_systems(PostUpdate, init_boxid_system.after(IntegrationSet::EndIntegration));
        app.add_systems(PostUpdate, apply_collisions_system.after(IntegrationSet::EndIntegration).after(init_boxid_system));
    }
}