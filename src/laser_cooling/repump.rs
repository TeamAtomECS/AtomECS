//! Handling of dark states and repumping

use crate::laser_cooling::photons_scattered::TotalPhotonsScattered;
use bevy::prelude::*;
use rand;
use rand::Rng;

use super::transition::TransitionComponent;

/// Marks an atom as being in a dark state
#[derive(Component)]
pub struct Dark;

/// Enables the possiblity to loose atoms into dark states
#[derive(Resource)]
pub struct RepumpLoss {
    /// Chance in the range [0,1] that an atom is depumped after scattering a photon.
    pub depump_chance: f64,
}

impl RepumpLoss {
    pub fn if_loss(&self, number_scattering_events: f64) -> bool {
        let mut rng = rand::rng();
        let result: f64 = rng.random_range(0.0..1.0);
        result < (1.0 - self.depump_chance).powf(number_scattering_events)
    }
}

/// Checks if an atom transitions into a dark state during the current
/// simulation step if a [RepumpLoss] resource has been added to the simulation.
pub fn make_atoms_dark<T: TransitionComponent>(
    repump_opt: Option<Res<RepumpLoss>>,
    atom_query: Query<(Entity, &TotalPhotonsScattered<T>)>,
    mut commands: Commands,
) {
    match repump_opt {
        None => (),
        Some(repump) => {
            for (ent, num) in atom_query.iter() {
                if repump.if_loss(num.total) {
                    commands.entity(ent).insert(Dark {});
                }
            }
        }
    }
}
