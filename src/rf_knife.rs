//! An 'RF Knife' is an oscillating magnetic field applied to remove atoms from the trap by driving transitions between atomic states.
//!
//! In this implementation, the RF knife removes atoms from the trap when the magnetic splitting is larger than the RF knife frequency.
//! Two removal methods are available. One destroys atoms and removes them from the simulation

#![allow(non_snake_case)]

use crate::atom::{Atom, Force, Mass, Position, Velocity};
use crate::destructor::ToBeDestroyed;
use crate::initiate::NewlyCreated;
use crate::integrator::OldForce;
use crate::magnetic::force::MagneticDipole;
use crate::magnetic::MagneticFieldSampler;
use crate::ramp::Lerp;
use nalgebra::Vector3;
use rayon::prelude::*;
use specs::ParJoin;
use specs::{
    Builder, Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System,
    WriteExpect, WriteStorage,
};

#[derive(Clone, Lerp)]
pub struct RFKnife {
    /// Frequency of the RF Knife in units of MHz.
    pub frequency: f64,
    /// Value of `g_F \mu_B`, in units of MHz / Gauss. This should really be a per-atom property, and will be moved there in the future.
    pub gFuB: f64,
}

impl RFKnife {
    pub fn method(&self) -> AtomRemovalMethod {
        AtomRemovalMethod::Resample
    }
}

pub enum AtomRemovalMethod {
    Destroy,
    Resample,
}

impl Component for RFKnife {
    type Storage = HashMapStorage<Self>;
}

pub struct ApplyRFKnifeSystem;
impl<'a> System<'a> for ApplyRFKnifeSystem {
    type SystemData = (
        WriteStorage<'a, RFKnife>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, MagneticFieldSampler>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteExpect<'a, crate::collisions::CollisionParameters>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );
    fn run(
        &mut self,
        (knives,atom, samplers, mut pos, mut vel, mut collisions, entities, updater): Self::SystemData,
    ) {
        for knife in (&knives).join() {
            match knife.method() {
                AtomRemovalMethod::Destroy => {
                    (&atom, &samplers, &entities).par_join().for_each(
                        |(_atom, sampler, entity)| {
                            let b_gauss = sampler.magnitude * 1e4; //sampler.field is in Tesla

                            let zeeman_splitting_mhz = (knife.gFuB * b_gauss).abs();
                            if zeeman_splitting_mhz > knife.frequency {
                                updater.insert(entity, ToBeDestroyed);
                            }
                        },
                    );
                }
                AtomRemovalMethod::Resample => {
                    // Get all positions/velocities of atoms.
                    // TODO: also copy old force
                    let pos_vel: Vec<(Vector3<f64>, Vector3<f64>)> = (&atom, &pos, &vel)
                        .join()
                        .map(|(_atom, pos, vel)| (pos.pos.clone(), vel.vel.clone()))
                        .collect();

                    let mut total_atoms = pos_vel.len() as f64 * collisions.macroparticle;

                    (&atom, &samplers, &mut pos, &mut vel).join().for_each(
                        |(_atom, sampler, mut position, mut velocity)| {
                            let b_gauss = sampler.magnitude * 1e4; //sampler.field is in Tesla

                            let zeeman_splitting_mhz = (knife.gFuB * b_gauss).abs();

                            if zeeman_splitting_mhz > knife.frequency {
                                // TODO: Remove duplication by breaking this into two separate systems - one which resamples, the other which marks atoms for removal by evap.
                                use rand::seq::SliceRandom;
                                let mut rng = rand::thread_rng();
                                let (chosen_pos, chosen_vel) = pos_vel.choose(&mut rng).unwrap();
                                position.pos = *chosen_pos;
                                velocity.vel = *chosen_vel;
                                total_atoms = total_atoms - collisions.macroparticle;
                            }
                        },
                    );

                    collisions.macroparticle = total_atoms / (pos_vel.len() as f64);
                }
            }
        }
    }
}

/// Resamples the atomic distribution during evaporative cooling.
///
/// Splits every atom in half. Uses the symmetry of the trap to 'reflect' each clone.
pub struct ResampleAtomsSystem;
impl<'a> System<'a> for ResampleAtomsSystem {
    type SystemData = (
        ReadStorage<'a, Atom>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Force>,
        ReadStorage<'a, OldForce>,
        ReadStorage<'a, Mass>,
        Entities<'a>,
        WriteExpect<'a, crate::collisions::CollisionParameters>,
        Read<'a, LazyUpdate>,
    );
    fn run(
        &mut self,
        (
            atoms,
            positions,
            velocities,
            forces,
            old_forces,
            masses,
            entities,
            mut collision_params,
            updater,
        ): Self::SystemData,
    ) {
        let atom_number = (&atoms, &positions).join().count();

        if atom_number > 4_000 {
            return;
        }

        (
            &atoms,
            &positions,
            &velocities,
            &forces,
            &old_forces,
            &masses,
        )
            .join()
            .for_each(|(_atom, position, velocity, _force, _old_force, mass)| {
                let pos = &position.pos;
                let vel = &velocity.vel;
                updater
                    .create_entity(&entities)
                    .with(Position {
                        pos: Vector3::new(-pos[0], -pos[1], pos[2]), // we assume a cylindrical symmetry to the trap here
                    })
                    .with(mass.clone())
                    .with(Force::new())
                    .with(Velocity {
                        vel: Vector3::new(-vel[0], -vel[1], vel[2]),
                    })
                    .with(MagneticDipole { mFgF: 0.5 }) // not general, obviously - actually quite hard to rewrite this in a general way
                    //we'll get to it, maybe by porting over to bevy
                    .with(OldForce::default())
                    .with(Atom {})
                    .with(NewlyCreated)
                    .build();
            });

        collision_params.macroparticle = collision_params.macroparticle / 2.0;
    }
}

pub mod tests {

    #[allow(unused_imports)]
    use super::*;
    extern crate specs;
    #[allow(unused_imports)]
    use specs::{Builder, Entity, RunNow, World};
    extern crate nalgebra;
    #[allow(unused_imports)]
    use crate::atom::{Atom, Position};
    #[allow(unused_imports)]
    use crate::integrator::{Step, Timestep};
    #[allow(unused_imports)]
    use nalgebra::Vector3;

    // #[test]
    // fn test_rf_knife_system(){
    //     let mut test_world = World::new();
    //     test_world.register::<Position>();

    //     test_world
    //         .create_entity()
    //         .with(RFKnife{radius: 1.0})
    //         .build();

    //     let test_atom1 = test_world.create_entity()
    //         .with(Atom)
    //         .with(Position{
    //             pos: Vector3::new(0.0, 0.0, 0.0)
    //         })
    //         .build();

    //     let test_atom2 = test_world.create_entity()
    //         .with(Atom)
    //         .with(Position{
    //             pos: Vector3::new(2.0, 0.0, 0.0)
    //         })
    //         .build();

    //     let mut system = ApplyRFKnifeSystem;
    //     system.run_now(&test_world.res);
    //     test_world.maintain();

    //     let positions = test_world.read_storage::<Position>();
    //     assert_eq!(positions.get(test_atom1).is_none(), false);
    //     assert_eq!(positions.get(test_atom2).is_none(), true);
    // }
}
