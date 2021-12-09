//! Handling of dark states and repumping

use rand;
extern crate specs;
use crate::laser_cooling::photons_scattered::TotalPhotonsScattered;
use rand::Rng;
use specs::{Component, Entities, LazyUpdate, Read, ReadStorage, System, VecStorage};

/// Marks an atom as being in a dark state
pub struct Dark;

impl Component for Dark {
    type Storage = VecStorage<Self>;
}

/// Enables the possiblity to loose atoms into dark states
pub struct RepumpLoss {
    /// Chance in the range [0,1] that an atom is depumped after scattering a photon.
    pub depump_chance: f64,
}

impl RepumpLoss {
    pub fn if_loss(&self, number_scattering_events: f64) -> bool {
        let mut rng = rand::thread_rng();
        let result: f64 = rng.gen_range(0.0..1.0);
        result < (1.0 - self.depump_chance).powf(number_scattering_events)
    }
}

/// Checks if an atom transitions into a dark state during the current
/// simulation step if a `RepumpLoss` component has been initialized.
pub struct RepumpSystem;

impl<'a> System<'a> for RepumpSystem {
    type SystemData = (
        Option<Read<'a, RepumpLoss>>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, TotalPhotonsScattered>,
        Entities<'a>,
    );
    fn run(&mut self, (repump_opt, lazy, num, ent): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        match repump_opt {
            None => (),
            Some(repump) => {
                (&ent, &num).par_join().for_each(|(ent, num)| {
                    if repump.if_loss(num.total) {
                        lazy.insert(ent, Dark {})
                    }
                });
            }
        }
    }
}
