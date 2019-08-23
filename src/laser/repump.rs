extern crate specs;
extern crate rand;
use rand::Rng;
use specs::{Component,VecStorage};

pub struct Dark;

impl Component for Dark{
    type Storage = VecStorage<Self>;
}

pub struct RepumpLoss {
    pub proportion: f64,
}

impl RepumpLoss{
    pub fn if_loss(&self) -> bool{
        let mut rng = rand::thread_rng();
		let result = rng.gen_range(0.0, 1.0);
        return (result < self.proportion)
    }
}


