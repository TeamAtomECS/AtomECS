use crate::atom::AtomInfo;
extern crate specs;

use crate::integrator::Timestep;
use crate::laser::LaserSamplers;
use crate::force::Force;
use specs::{
    Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, VecStorage,
    WriteStorage,ReadExpect
};
use crate::laser::repump::*;
use crate::constant;

let threshold = 0.99;

pub struct energy_level {
    pub levels: Vec<String>,
    pub energy: Vec<i64>,
    pub current_level: Vec<i64>,
    ///the probability of having the atom in corresponding states
    pub angular_momentum: Vec<i64>,
    pub dark_state: Vec<bool>,
}

impl Component for energy_level {
    type Storage = VecStorage<Self>;
}

pub struct EnergyLevelOption;

pub struct AtomTransistionRateSystem;

impl<'a> System<'a> for AtomTransistionRateSystem {
    type SystemData = (
        ReadStorage<'a,energy_level>,
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, Dark>,
        ReadStorage<'a, NumberScattered>,
        ReadExpect<'a,EnergyLevelOption>,
        WriteStorage<'a,Force>
    );

    fn run(&mut self, (samplers, _atom, timestep, _dark, mut number,lvl_option,mut force): Self::SystemData) {
        let mut energyleveloption = false;
        match lvl_opt {
            None => (),
            Some(_rand) => {
                energyleveloption = true;
            }
        }
        if lvl_opt{
            for (samplers, _, atom_info, (), num,force) in
                (&samplers, &_atom, &atom_info, !&_dark, &mut number,&mut force).join()
            {
                for i in range(len(energy_level.current_level)){
                    for j in range(len(energy_level.current_level)){
                        ///for each pair of state
                        let mut trans_prob = 0.0;
                        /// TODO calculate the transition probablity
                        
                        if trans_prob*timestep.delta >0.1{
                            ///TODO equalibium state calculation
                            /// involve multiple state, so probablity a equation solver is needed
                            
                        }
                        else{
                            ///TODO simulating the energy levels
                            /// probably is not needed anymore
                        }
                    }
                }
            }
        }
    }
}

pub struct IdentifyDarkSystem;

impl<'a> System<'a> for IdentifyDarkSystem{
    type SystemData =(
        ReadStorage<'a,energy_level>,
        ReadStorage<'a,atom>,
        WriteStorage<'a,Dark>,
        ReadExpect<'a,EnergyLevelOption>
        Read<'a, LazyUpdate>,
        Entities<'a>,
    )
    /// identify atom within the pre defined "dark state" to save computation time
    fn run(&mut self, (energylevel, _atom, mut dark, lvl_opt,updater,ent): Self::SystemData) {
        let mut energyleveloption = false;
        match lvl_opt {
            None => (),
            Some(_rand) => {
                energyleveloption = true;
            }
        }
        if energyleveloption{
            for (level,mag_sampler _, (),mut force,ent) in
        (&energylevel,&_atom,&mut dark,&mut force,&ent).join(){
                for i in range(len(level.dark_state)){
                    if (level.current_level[i]> threshold) || level.dark_state[i]{
                        updater.insert(ent,Dark);
                    }
                }
            }
        }
    }
}