extern crate rand;
extern crate specs;
use crate::laser::force::NumberScattered;
use rand::Rng;
use specs::{Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage};

pub struct Dark;

impl Component for Dark {
    type Storage = VecStorage<Self>;
}

pub struct RepumpLoss {
    /// Chance in the range [0,1] that an atom is depumped after scattering a photon.
    pub depump_chance: f64,
}

impl RepumpLoss {
    pub fn if_loss(&self, number_scattering_events: f64) -> bool {
        let mut rng = rand::thread_rng();
        let result = rng.gen_range(0.0, 1.0);
        return result < (1.0 - self.depump_chance).powf(number_scattering_events);
    }
}

pub struct RepumpSystem;

impl<'a> System<'a> for RepumpSystem {
    type SystemData = (
        Option<Read<'a, RepumpLoss>>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, NumberScattered>,
        Entities<'a>,
    );
    fn run(&mut self, (repump_opt, lazy, num, ent): Self::SystemData) {
        match repump_opt {
            None => (),
            Some(repump) => {
                for (ent, num) in (&ent, &num).join() {
                    if repump.if_loss(num.value) {
                        lazy.insert(ent, Dark {})
                    }
                }
            }
        }
    }
}
