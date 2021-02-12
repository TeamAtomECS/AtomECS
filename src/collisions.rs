// Implement s wave scattering of atoms

extern crate multimap;
use multimap::MultiMap;
use crate::atom::{Mass,Position,Velocity};
use crate::constant;
use nalgebra::Vector3;
use specs::{Join, Read, ReadStorage, System, WriteStorage};
use rand::Rng;


/// A resource that indicates that the simulation should apply scattering
pub struct ApplyCollisionsOption;

/// This system applies scattering to atoms
/// Uses spatial partitioning for faster calculation
/// 
pub struct ApplyCollisionsSystem;
impl<'a> System<'a> for ApplyCollisionsSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        Option<Read<'a,ApplyCollisionsOption>>,
    );

    fn run(&mut self, (positions, mut velocities, collisions_option): Self::SystemData ){
        match collisions_option {
            None => (),//(println!("No collisions option enabled")),
            Some(_) => {
                //make hash table - dividing space up into grid
                let mut map = MultiMap::new();
                let N: i64 = 100; // number of boxes per side
                let width: f64 = 5e-6; // width of each box
                let mut bin_list = Vec::new();


                for (position,velocity) in (&positions, &mut velocities).join() {
                    //Assume that atoms that leave the grid are too sparse to collide, so disregard them
                    let bound = (N as f64)/2.0 * width;
                    if position.pos[0].abs() > bound {
                        continue
                    } else if position.pos[1].abs() > bound {
                        continue
                    } else if position.pos[2].abs() > bound {
                        continue
                    }

                    //centre grid on origin
                    let xp = ((position.pos[0]-((N as f64)/2.0).floor())/width).floor() as i64;
                    let yp = ((position.pos[1]-((N as f64)/2.0).floor())/width).floor() as i64;
                    let zp = ((position.pos[2]-((N as f64)/2.0).floor())/width).floor() as i64;

                    //convert position to box key
                    let rounded = Vector3::new(xp,yp,zp);
                    let key = rounded[0]+N*rounded[1]+N.pow(2)*rounded[2];
                    
        
                    //insert atom velocity into hash with that key
                    map.insert(key, velocity);
                    bin_list.push(key);  
                    
                    }

                let mut bin_ids = bin_list.clone();
                bin_ids.sort();
                bin_ids.dedup();


                for key in &bin_ids {
                    let result = map.get_vec_mut(key);

                    let collision_chance = 0.1;
                    let mut rng = rand::thread_rng();
                    
                    match result {
                        Some(vels) =>
                            for i in 0..vels.len()-1{
                                for j in i+1..vels.len()-1{
                                

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
                        
                        None => ()//println!("No velocities found")
                        }   
                    }
                }   
            }
        }
    }