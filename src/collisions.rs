//! Implement s wave scattering of atoms

extern crate multimap;
use crate::atom::{Position, Velocity};
use crate::integrator::Timestep;
use crate::constant::{PI};
use hashbrown::HashMap;
use rand::Rng;
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, VecStorage,
    WriteStorage,
};
use nalgebra::Vector3;


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
                    if boxid.id == i64::MAX{
                        continue
                    } else {
                    map.entry(boxid.id).or_default().push(velocity);
                    }
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
                    let num_collisions_expected = (params.macroparticle * (number as f64)).powi(2) * params.sigma * vbar * t.delta * width.powi(-3);


                    // loop over number of collisions happening
                    // if number is low (<0.5) treat it as the probability of one total collision occurring
                    // otherwise, round to nearest integer and select that many pairs to randomly 
                    let mut num_collisions: i32;
                    
                    // println!("num_collisions_expected:{}, number:{}", num_collisions_expected,number);

                    if num_collisions_expected <= 0.5 {
                        let p = rng.gen::<f64>();
                        if p < num_collisions_expected {
                            let idx = rng.gen_range(0, number - 1) as usize;
                            let mut idx2 = rng.gen_range(0, number - 1) as usize;
                            if idx2 == idx{
                                idx2 = idx+1;
                            }

                            let v1 = velocities[idx].vel;
                            let v2 = velocities[idx2].vel;


                            // Randomly modify velocities in CoM frame, conserving energy & momentum
                            let vcm = 0.5*(v1+v2);
                            let energy = 0.5*( (v1-vcm).norm_squared() + (v2-vcm).norm_squared());

                            let cos_theta: f64 = rng.gen_range(-1.0,1.0);
                            let sin_theta: f64 = (1.0-cos_theta.powi(2)).sqrt();
                            let phi: f64 = rng.gen_range(0.0, 2.0*PI);

                            let v_prime = Vector3::new(energy.sqrt()*sin_theta*phi.cos(), energy.sqrt()*sin_theta*phi.sin(),energy.sqrt()*cos_theta);
                            velocities[idx].vel = vcm + v_prime;
                            velocities[idx2].vel = vcm - v_prime;
                        } 
                    } else {
                        num_collisions = num_collisions_expected.round() as i32;

                        if num_collisions > 100000 as i32{

                            num_collisions = 100000  as i32;
                        }

                        for _i in 1..num_collisions {

                            let idx = rng.gen_range(0, number - 1) as usize;
                            let mut idx2 = rng.gen_range(0, number - 1) as usize;
                            if idx2 == idx{
                                idx2 = idx+1;
                            }

                            let v1 = velocities[idx].vel;
                            let v2 = velocities[idx2].vel;


                            // Randomly modify velocities in CoM frame, conserving energy & momentum
                            let vcm = 0.5*(v1+v2);
                            let energy = 0.5*( (v1-vcm).norm_squared() + (v2-vcm).norm_squared());

                            let cos_theta: f64 = rng.gen_range(-1.0,1.0);
                            let sin_theta: f64 = (1.0-cos_theta.powi(2)).sqrt();
                            let phi: f64 = rng.gen_range(0.0, 2.0*PI);

                            let v_prime = Vector3::new(energy.sqrt()*sin_theta*phi.cos(), energy.sqrt()*sin_theta*phi.sin(),energy.sqrt()*cos_theta);
                            velocities[idx].vel = vcm + v_prime;
                            velocities[idx2].vel = vcm - v_prime;
                        }
                    }
                }
                    
                });
            }
        }
    }
}

// fn do_collision(mut vel1: Velocity, mut vel2: Velocity) -> (Velocity, Velocity) {
//     let mut rng = rand::thread_rng();
//     let v1 = vel1.vel;
//     let v2 = vel2.vel;


//     // Randomly modify velocities in CoM frame, conserving energy & momentum
//     let vcm = 0.5*(v1+v2);
//     let energy = 0.5*( (v1-vcm).norm().powi(2) + (v2-vcm).norm().powi(2));

//     let theta: f64 = rng.gen_range(0.0,2.0*PI);
//     let phi: f64 = rng.gen_range(0.0, 2.0*PI);

//     let v_prime = Vector3::new(energy.sqrt()*theta.sin()*phi.cos(), energy.sqrt()*theta.sin()*phi.sin(),energy.sqrt()*theta.cos());
//     vel1.vel = vcm + v_prime;
//     vel2.vel = vcm - v_prime;

//     (vel1,vel2)
// }