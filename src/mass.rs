use crate::atom::Mass;
extern crate rand;
use rand::Rng;
extern crate specs;

use specs::{Component, VecStorage};
use serde::{Deserialize,Serialize};

#[derive(Deserialize,Serialize,Clone)]
pub struct MassPercentage{
    pub atommass:f64,
    pub percentage:f64,
}

#[derive(Deserialize,Serialize,Clone)]
pub struct MassArchetype{
    pub massdistribution : Vec<MassPercentage>,
}

impl Component for MassArchetype{
    type Storage = VecStorage<Self>;
}

impl MassArchetype{
    pub fn normalise(&mut self){
        let mut total = 0.;
        for masspercent in self.massdistribution.iter(){
            total = total + masspercent.percentage;
        }
        
        for mut masspercent in &mut self.massdistribution{
            masspercent.percentage = masspercent.percentage/ total;
        }
    }
    pub fn get_mass(&self) -> Mass{
        let mut level = 0.;
        let mut rng = rand::thread_rng();
	    let luck = rng.gen_range(0.0, 1.0);
        let mut finalmass = 0.;
        for masspercent in self.massdistribution.iter(){
            level = level + masspercent.percentage;
            if level> luck{
                return Mass{value : masspercent.atommass}
            }
            finalmass = masspercent.atommass;
        }
        return Mass{value : finalmass}
    }
}