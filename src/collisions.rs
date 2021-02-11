// Implement s wave scattering of atoms

extern crate multimap;
use multimap::MultiMap;
use crate::atom::{Mass,Position,Velocity};
use crate::constant;
use nalgebra::Vector3;
use specs::{Join, Read, ReadStorage, System, WriteStorage};
use rand::Rng;


/// This system applies scattering to atoms
/// Uses spatial partitioning for faster calculation
/// 
pub struct ApplyCollisionsSystem;
impl<'a> System<'a> for ApplyCollisionsSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (positions, mut velocities): Self::SystemData ){
        
            //make hash table - dividing space up into grid
            let mut map = MultiMap::new();
            let N: i32 = 200; // number of boxes per side
            let width: f64 = 2e-6; // width of each box
            let mut bin_list = Vec::new();


            for (position,velocity) in (&positions, &mut velocities).join() {

                //convert position to box key
                let rounded = Vector3::new((position.pos[0]/width).floor() as i32,(position.pos[1]/width).floor() as i32,(position.pos[2]/width).floor() as i32);
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
                //let mut vels: Vec<_> = (&mut velocities).join().collect();

                let collision_chance = 0.0;
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
                    
                    None => println!("No velocities found.")
                    }   

                }
        }

    }