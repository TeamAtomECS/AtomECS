//! Implement s wave scattering of atoms

extern crate multimap;
use crate::atom::{Position, Velocity};
use crate::integrator::Timestep;
use multimap::MultiMap;
use rand::Rng;
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, VecStorage,
    WriteStorage,
};

/// A resource that indicates that the simulation should apply scattering
pub struct ApplyCollisionsOption;

/// Component that marks which box an atom is in for spatial partitioning
///

pub struct BoxID {
    /// ID of the box
    pub id: i64,
}
impl Component for BoxID {
    type Storage = VecStorage<Self>;
}

/// This system applies scattering to atoms
/// Uses spatial partitioning for faster calculation
///
pub struct ApplyCollisionsSystem;
impl<'a> System<'a> for ApplyCollisionsSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        Option<Read<'a, ApplyCollisionsOption>>,
        ReadExpect<'a, Timestep>,
        Entities<'a>,
        WriteStorage<'a, BoxID>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (positions, mut velocities, collisions_option, t, entities, boxids, updater): Self::SystemData,
    ) {
        match collisions_option {
            None => (), //(println!("No collisions option enabled")),
            Some(_) => {
                //make hash table - dividing space up into grid
                let mut map = MultiMap::new();
                let N: i64 = 50; // number of boxes per side
                let width: f64 = 3e-6; // width of each box
                let macroparticle = 1e5; // number of real particles each simulation particle represents for purposes of scaling collision physics
                let mut bin_list = Vec::new();

                use rayon::prelude::*;
                use specs::ParJoin;
                // build list of ids for each atom
                (&positions, &entities)
                    .par_join()
                    .for_each(|(position, ent)| {
                        //Assume that atoms that leave the grid are too sparse to collide, so disregard them
                        //We'll assign them the max value of i64, and then check for this value when we do a collision and ignore them
                        let bound = (N as f64) / 2.0 * width;

                        let id: i64;
                        if position.pos[0].abs() > bound {
                            id = i64::MAX;
                        } else if position.pos[1].abs() > bound {
                            id = i64::MAX;
                        } else if position.pos[2].abs() > bound {
                            id = i64::MAX;
                        } else {
                            //centre grid on origin
                            let xp = ((position.pos[0] - ((N as f64) / 2.0).floor()) / width)
                                .floor() as i64;
                            let yp = ((position.pos[1] - ((N as f64) / 2.0).floor()) / width)
                                .floor() as i64;
                            let zp = ((position.pos[2] - ((N as f64) / 2.0).floor()) / width)
                                .floor() as i64;

                            //convert position to box id
                            id = xp + N * yp + N.pow(2) * zp;
                        }

                        updater.insert(ent, BoxID { id: id });
                    });

                //insert atom velocity into hash

                for (velocity, boxid) in (&mut velocities, &boxids).join() {
                    map.insert(boxid.id, velocity);
                    bin_list.push(boxid.id);
                }

                bin_list.sort();
                bin_list.dedup();

                (&bin_list).par_iter().for_each(|id| {
                    let result = map.get_vec_mut(&id);

                    let mut rng = rand::thread_rng();
                    match result {
                        Some(vels) => {
                            for i in 0..vels.len() - 1 {
                                for j in i + 1..vels.len() - 1 {
                                    //use relative velocity to make collisions velocity dependent
                                    let vrel = (vels[i].vel - vels[j].vel).norm();
                                    let sigma = 3.5e-16; // cross section for Rb

                                    let collision_chance = macroparticle * sigma * vrel * t.delta;
                                    let p = rng.gen::<f64>();

                                    if p < collision_chance {
                                        let v1 = vels[i].vel;
                                        let v2 = vels[j].vel;

                                        let temp = v1;

                                        vels[i].vel = v2;
                                        vels[j].vel = temp;
                                    }
                                }
                            }
                        }

                        None => (), //println!("No velocities found")
                    }
                });
            }
        }
    }
}
