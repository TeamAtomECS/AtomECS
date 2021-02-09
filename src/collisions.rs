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
pub struct ApplyCollisionsSystem;
impl<'a> System<'a> for ApplyCollisionsSystem {
    type SystemData = (
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Mass>,
        ReadStorage<'a, Position>,
        Option<Read<'a, ApplySWaveOption>,
    );

    fn run(&mut self, (positions, mut velocities): Self::SystemData ){
        
            //make hash table - dividing space up into grid
            let mut map = MultiMap::new();
            let N = 10; // number of boxes per side
            let width = 50e-6; // width of each box

        for (position,velocity) in (&positions, &velocities) {

            //convert position to box key
            let rounded = Vector3::new((position.pos[0]/width).floor(),(position.pos[1]/width).floor(),(position.pos[2]/width).floor());
            key = rounded[0]+N*rounded[1]+N.powf(2)*rounded[2];

            //insert atom velocity into hash with that key
            map.insert(key, velocity)
            
        }

        for key in &map.keys {

            vels = map.get(key);

            for mut vel1 in &vels{
                for mut vel2 in &vels{
                    if vel1 == vel2{
                        break
                    } else {
                        vel1, vel2 = do_collision(vel1,vel2);
                    }
                }
            }
        }

    }
}


fn do_collision(vel1: &Velocity,vel2: &Velocity) -> (Velocity, Velocity) {
    let collision_chance = 0.1;

    let mut rng = rand::thread_rng();

    let p = rng.gen::<f64>();

    if p < collision_chance{
        let vel1_new = vel2;
        let vel2_new = vel1;
    } else {
        let vel1_new = vel1;
        let vel2_new = vel2;
    }

    vel1_new, vel2_new

}