//! Implement s wave scattering of atoms

extern crate multimap;
use crate::atom::{Position, Velocity};
use crate::integrator::Timestep;
use hashbrown::HashMap;
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

/// Resource for defining collision relevant paramaters like macroparticle number, box width and number of boxes
/// 
pub struct CollisionParameters{
    /// number of real particles one simulation particle represents for collisions
    pub macroparticle: f64,
    //number of boxes per side in spatial binning
    pub box_number: i64,
    //width of one box in m
    pub box_width: f64,
    // collisional cross section of atoms (assuming only one species)
    pub sigma: f64,
}

/// This system applies scattering to atoms
/// Uses spatial partitioning for faster calculation
///
pub struct ApplyCollisionsSystem;
impl<'a> System<'a> for ApplyCollisionsSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, crate::atom::Atom>,
        WriteStorage<'a, Velocity>,
        Option<Read<'a, ApplyCollisionsOption>>,
        ReadExpect<'a, Timestep>,
        Entities<'a>,
        WriteStorage<'a, BoxID>,
        Read<'a, LazyUpdate>,
        ReadExpect<'a, CollisionParameters>,
    );

    fn run(
        &mut self,
        (positions, atoms, mut velocities, collisions_option, t, entities, mut boxids, updater,params): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        match collisions_option {
            None => (),
            Some(_) => {
                //make hash table - dividing space up into grid
                let mut map: HashMap<i64, Vec<&mut Velocity>> = HashMap::new();
                let n: i64 = params.box_number; // number of boxes per side
                let width: f64 = params.box_width; // width of each box in m

                // Get all atoms which do not have boxIDs
                for (entity, _, _) in (&entities, &atoms, !&boxids).join() {
                    updater.insert(entity, BoxID { id: 0 });
                }

                // build list of ids for each atom
                (&positions, &mut boxids)
                    .par_join()
                    .for_each(|(position, mut boxid)| {
                        //Assume that atoms that leave the grid are too sparse to collide, so disregard them
                        //We'll assign them the max value of i64, and then check for this value when we do a collision and ignore them
                        let bound = (n as f64) / 2.0 * width;

                        let id: i64;
                        if position.pos[0].abs() > bound {
                            id = i64::MAX;
                        } else if position.pos[1].abs() > bound {
                            id = i64::MAX;
                        } else if position.pos[2].abs() > bound {
                            id = i64::MAX;
                        } else {
                            //centre grid on origin
                            let xp = ((position.pos[0] - ((n as f64) / 2.0).floor()) / width)
                                .floor() as i64;
                            let yp = ((position.pos[1] - ((n as f64) / 2.0).floor()) / width)
                                .floor() as i64;
                            let zp = ((position.pos[2] - ((n as f64) / 2.0).floor()) / width)
                                .floor() as i64;

                            //convert position to box id
                            id = xp + n * yp + n.pow(2) * zp;
                        }
                        boxid.id = id;
                    });

                //insert atom velocity into hash
                for (velocity, boxid) in (&mut velocities, &boxids).join() {
                    map.entry(boxid.id).or_default().push(velocity);
                }
                map.par_values_mut().for_each(|velocities| {
                    let mut rng = rand::thread_rng();
                    let number = velocities.len() as i32;

                    if number <= 1{

                    } else{
                    
                    // calculate average speed (not velocity)
                    // average velocity will be close to zero since many particles are moving in different directions
                    // we just want a typical speed for calculating collision probability
                    let mut vsum = 0.0;
                    for i in 0..(number-1) as usize{
                        vsum = vsum + velocities[i].vel.norm();
                    }

                    let vbar = vsum / number as f64;
                    // number of collisions is N*n*sigma*v*dt, where n is atom density and N is atom number
                    let num_collisions_expected = (params.macroparticle * (number as f64)).powf(2.0) * params.sigma * vbar * t.delta * width.powf(-3.0);

                    // loop over number of collisions happening
                    // if number is low (<0.5) treat it as the probability of one total collision occurring
                    // otherwise, round to nearest integer and select that many pairs to randomly 
                    // if expected number of collisions is higher than number of simulation particles, just loop over all the particles
                    let mut num_collisions: i32;

                    if num_collisions_expected <= 0.5 {
                        let p = rng.gen::<f64>();
                        if p < num_collisions_expected {
                            let idx = rng.gen_range(0, number - 1) as usize; //note gen_range only goes up to number-2 here

                            let v1 = velocities[idx].vel;
                            let v2 = velocities[idx+1].vel;

                            let temp = v1;

                            velocities[idx].vel = v2;
                            velocities[idx+1].vel = temp;

                        } 
                    } else {
                        num_collisions = num_collisions_expected.round() as i32;

                        if num_collisions > number {
                            num_collisions = number;
                        }

                        for _i in 1..num_collisions {
                            let idx = rng.gen_range(0, number - 1) as usize;

                            let v1 = velocities[idx].vel;
                            let v2 = velocities[idx+1].vel;

                            let temp = v1;

                            velocities[idx].vel = v2;
                            velocities[idx+1].vel = temp;
                        }
                    }
                }
                    
                });
            }
        }
    }
}