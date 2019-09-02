
extern crate rand;
extern crate specs;
use crate::laser::force::NumberKick;
use rand::Rng;
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, VecStorage,
};

pub struct Dark;

impl Component for Dark {
    type Storage = VecStorage<Self>;
}

pub struct RepumpLoss {
    pub proportion: f64,
}

impl RepumpLoss {
    pub fn if_loss(&self) -> bool {
        let mut rng = rand::thread_rng();
        let result = rng.gen_range(0.0, 1.0);
        return result < self.proportion;
    }
}

pub struct RepumpSystem;

impl<'a> System<'a> for RepumpSystem {
    type SystemData = (
        ReadExpect<'a, RepumpLoss>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, NumberKick>,
        Entities<'a>,
    );
    fn run(&mut self, (repump, lazy, num, ent): Self::SystemData) {
        for (ent, num) in (&ent, &num).join() {
            for _i in 0..num.value {
                if repump.if_loss() {
                    lazy.insert(ent, Dark {})
                }
            }
        }
    }
}

