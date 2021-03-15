//! Implement s wave scattering of atoms

extern crate multimap;
use crate::atom::{Position, Velocity};
use crate::constant::PI;
use crate::integrator::Timestep;
use hashbrown::HashMap;
use nalgebra::Vector3;
use rand::Rng;
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, VecStorage,
    WriteExpect, WriteStorage,
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
pub struct CollisionParameters {
    /// number of real particles one simulation particle represents for collisions
    pub macroparticle: f64,
    //number of boxes per side in spatial binning
    pub box_number: i64,
    //width of one box in m
    pub box_width: f64,
    // collisional cross section of atoms (assuming only one species)
    pub sigma: f64,
    //total number of collisions overall
    pub num_total: i64,
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
        WriteExpect<'a, CollisionParameters>,
    );

    fn run(
        &mut self,
        (
            positions,
            atoms,
            mut velocities,
            collisions_option,
            t,
            entities,
            mut boxids,
            updater,
            mut params,
        ): Self::SystemData,
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
                        boxid.id = pos_to_id(position.pos, n, width);
                    });

                //insert atom velocity into hash
                for (velocity, boxid) in (&mut velocities, &boxids).join() {
                    if boxid.id == i64::MAX {
                        continue;
                    } else {
                        map.entry(boxid.id).or_default().push(velocity);
                    }
                }

                let mut collisions_vec: Vec<i64>;

                map.par_values_mut().for_each(|velocities| {
                    let mut rng = rand::thread_rng();
                    let number = velocities.len() as i32;

                    if number <= 1 {
                    } else {
                        // calculate average speed (not velocity)
                        // average velocity will be close to zero since many particles are moving in different directions
                        // we just want a typical speed for calculating collision probability
                        let mut vsum = 0.0;
                        for i in 0..(number - 1) as usize {
                            vsum = vsum + velocities[i].vel.norm();
                        }

                        let vbar = vsum / number as f64;
                        // number of collisions is N*n*sigma*v*dt, where n is atom density and N is atom number
                        let num_collisions_expected = (params.macroparticle * (number as f64))
                            .powi(2)
                            * params.sigma
                            * vbar
                            * t.delta
                            * width.powi(-3);

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
                                if idx2 == idx {
                                    idx2 = idx + 1;
                                }

                                let v1 = velocities[idx].vel;
                                let v2 = velocities[idx2].vel;
                                let (v1new, v2new) = do_collision(v1, v2);
                                velocities[idx].vel = v1new;
                                velocities[idx2].vel = v2new;
                            }
                        } else {
                            num_collisions = num_collisions_expected.round() as i32;

                            if num_collisions > 100000 as i32 {
                                num_collisions = 100000 as i32;
                            }

                            for _i in 0..num_collisions {
                                let idx = rng.gen_range(0, number - 1) as usize;
                                let mut idx2 = rng.gen_range(0, number - 1) as usize;
                                if idx2 == idx {
                                    idx2 = idx + 1;
                                }

                                let v1 = velocities[idx].vel;
                                let v2 = velocities[idx2].vel;
                                let (v1new, v2new) = do_collision(v1, v2);
                                velocities[idx].vel = v1new;
                                velocities[idx2].vel = v2new;
                            }
                            // println!(
                            //     "number: {}, actual number of collisions: {}",
                            //     number, num_collisions
                            // );
                        }
                    }
                });
            }
        }
    }
}

fn do_collision<'a>(mut v1: Vector3<f64>, mut v2: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let mut rng = rand::thread_rng();

    // Randomly modify velocities in CoM frame, conserving energy & momentum
    let vcm = 0.5 * (v1 + v2);
    let energy: f64 = 0.5 * ((v1 - vcm).norm().powi(2) + (v2 - vcm).norm().powi(2));

    let cos_theta: f64 = rng.gen_range(-1.0, 1.0);
    let sin_theta: f64 = (1.0 - cos_theta.powi(2)).sqrt();
    let phi: f64 = rng.gen_range(0.0, 2.0 * PI);

    let v_prime = Vector3::new(
        energy.sqrt() * sin_theta * phi.cos(),
        energy.sqrt() * sin_theta * phi.sin(),
        energy.sqrt() * cos_theta,
    );
    v1 = vcm + v_prime;
    v2 = vcm - v_prime;

    (v1, v2)
}

fn pos_to_id(pos: Vector3<f64>, n: i64, width: f64) -> i64 {
    //Assume that atoms that leave the grid are too sparse to collide, so disregard them
    //We'll assign them the max value of i64, and then check for this value when we do a collision and ignore them
    let bound = (n as f64) / 2.0 * width;

    let id: i64;
    if pos[0].abs() > bound {
        id = i64::MAX;
    } else if pos[1].abs() > bound {
        id = i64::MAX;
    } else if pos[2].abs() > bound {
        id = i64::MAX;
    } else {
        //centre grid on origin
        //grid cells run from [0, width), i.e include lower bound but exclude upper
        let xp = (pos[0] / width + 0.5 * (n as f64)).floor() as i64;
        let yp = (pos[1] / width + 0.5 * (n as f64)).floor() as i64;
        let zp = (pos[2] / width + 0.5 * (n as f64)).floor() as i64;
        //convert position to box id
        id = xp + n * yp + n.pow(2) * zp;
    }
    id
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::atom::{Atom, Force, Mass, Position, Velocity};
    #[allow(unused_imports)]
    use crate::ecs;
    #[allow(unused_imports)]
    use crate::ecs::AtomecsDispatcherBuilder;
    #[allow(unused_imports)]
    use crate::initiate::NewlyCreated;
    #[allow(unused_imports)]
    use crate::integrator::{
        Step, Timestep, VelocityVerletIntegratePositionSystem,
        VelocityVerletIntegrateVelocitySystem,
    };

    #[allow(unused_imports)]
    use nalgebra::Vector3;
    #[allow(unused_imports)]
    use specs::{Builder, Entity, RunNow, World};
    extern crate specs;

    #[test]
    fn test_pos_to_id() {
        let n: i64 = 10;
        let width: f64 = 2.0;

        let pos1 = Vector3::new(0.0, 0.0, 0.0);
        let pos2 = Vector3::new(1.0, 0.0, 0.0);
        let pos3 = Vector3::new(2.0, 0.0, 0.0);
        let pos4 = Vector3::new(9.9, 0.0, 0.0);
        let pos5 = Vector3::new(-9.9, 0.0, 0.0);
        let pos6 = Vector3::new(10.1, 0.0, 0.0);
        let pos7 = Vector3::new(-9.9, -9.9, -9.9);

        let id1 = pos_to_id(pos1, n, width);
        let id2 = pos_to_id(pos2, n, width);
        let id3 = pos_to_id(pos3, n, width);
        let id4 = pos_to_id(pos4, n, width);
        let id5 = pos_to_id(pos5, n, width);
        let id6 = pos_to_id(pos6, n, width);
        let id7 = pos_to_id(pos7, n, width);

        assert_eq!(id1, 555);
        assert_eq!(id2, 555);
        assert_eq!(id3, 556);
        assert_eq!(id4, 559);
        assert_eq!(id5, 550);
        assert_eq!(id6, i64::MAX);
        assert_eq!(id7, 0);
    }

    #[test]
    fn test_do_collision() {
        // do this test muliple times since there is a random element involved in do_collision
        for _i in 0..50 {
            let v1 = Vector3::new(0.5, 1.0, 0.75);
            let v2 = Vector3::new(0.2, 0.0, 1.25);
            //calculate energy and momentum before
            let ptoti = v1 + v2;
            let energyi = 0.5 * (v1.norm_squared() + v2.norm_squared());

            let (v1new, v2new) = do_collision(v1, v2);

            //energy and momentum after
            let ptotf = v1new + v2new;
            let energyf = 0.5 * (v1new.norm_squared() + v2new.norm_squared());

            assert!((ptoti - ptotf) <= Vector3::new(1e-6, 1e-6, 1e-6));
            assert!((energyi - energyf) <= 1e-6);
            assert_ne!(v1, v1new);
            assert_ne!(v2, v2new);
        }
    }

    #[test]
    fn test_collisions() {
        let mut test_world = World::new();

        ecs::register_components(&mut test_world);
        ecs::register_resources(&mut test_world);
        test_world.register::<NewlyCreated>();
        let mut atomecs_builder = AtomecsDispatcherBuilder::new();
        atomecs_builder.add_frame_initialisation_systems();
        atomecs_builder.add_systems();
        atomecs_builder
            .builder
            .add(ApplyCollisionsSystem, "collisions", &[]);
        atomecs_builder.add_frame_end_systems();

        let builder = atomecs_builder.builder;
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut test_world.res);

        let vel1 = Vector3::new(1.0, 0.0, 0.0);
        let vel2 = Vector3::new(-1.0, 0.0, 0.0);

        let pos1 = Vector3::new(-3.0, 0.0, 0.0);
        let pos2 = Vector3::new(3.0, 0.0, 0.0);

        //atom 1 to collide
        let atom1 = test_world
            .create_entity()
            .with(Velocity { vel: vel1 })
            .with(Position { pos: pos1 })
            .with(Atom)
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(NewlyCreated)
            .build();

        //atom2 to collide
        let atom2 = test_world
            .create_entity()
            .with(Velocity { vel: vel2 })
            .with(Position { pos: pos2 })
            .with(Atom)
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(NewlyCreated)
            .build();

        let dt = 1.0;
        test_world.add_resource(Timestep { delta: dt });
        test_world.add_resource(ApplyCollisionsOption);
        test_world.add_resource(CollisionParameters {
            macroparticle: 1.0,
            box_number: 10,
            box_width: 2.0,
            sigma: 10.0,
            num_total: 0,
        });

        for _i in 0..10 {
            dispatcher.dispatch(&mut test_world.res);
            test_world.maintain();
        }

        let velocities = test_world.read_storage::<Velocity>();
        let vel1new = velocities.get(atom1).expect("atom1 not found");
        let vel2new = velocities.get(atom2).expect("atom2 not found");

        let positions = test_world.read_storage::<Position>();
        let pos1new = positions.get(atom1).expect("atom1 not found");
        let pos2new = positions.get(atom2).expect("atom2 not found");

        assert_ne!(pos1, pos1new.pos);
        assert_ne!(pos2, pos2new.pos);

        assert_ne!(vel1 - vel1new.vel, Vector3::new(0.0, 0.0, 0.0));
        assert_ne!(vel2 - vel2new.vel, Vector3::new(0.0, 0.0, 0.0));
    }
}
