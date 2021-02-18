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
    );

    fn run(
        &mut self,
        (positions, atoms, mut velocities, collisions_option, t, entities, mut boxids, updater): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        match collisions_option {
            None => (),
            Some(_) => {
                //make hash table - dividing space up into grid
                let mut map: HashMap<i64, Vec<&mut Velocity>> = HashMap::new();
                let n: i64 = 50; // number of boxes per side
                let width: f64 = 5e-6; // width of each box
                let macroparticle = 1e5; // number of real particles each simulation particle represents for purposes of scaling collision physics

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
                    let number = velocities.len() - 1;

                    
                    for i in 0..number {
                        for j in i + 1..number {
                            //use relative velocity to make collisions velocity dependent
                            let vrel = (velocities[i].vel - velocities[j].vel).norm();
                            let sigma = 3.5e-16; // cross section for Rb
                            // chance of collision per atom is n*sigma*v*dt, where n is atom density
                            let collision_chance = macroparticle * (number as f64) * sigma * vrel * t.delta / width.powf(3.0);
                            let p = rng.gen::<f64>();

                            if p < collision_chance {
                                let v1 = velocities[i].vel;
                                let v2 = velocities[j].vel;

                                let temp = v1;

                                velocities[i].vel = v2;
                                velocities[j].vel = temp;
                            }
                        }
                    }
                });
            }
        }
    }
}
