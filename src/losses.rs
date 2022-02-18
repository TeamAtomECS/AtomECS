// Implement 2 and 3 body losses of atoms

extern crate multimap;
use crate::atom::Atom;
use crate::collisions::CollisionParameters;
use crate::destructor::ToBeDestroyed;
use crate::integrator::Timestep;
use crate::partition::{DensityHashmap, PartitionCell, PartitionParameters};
use rand::seq::index::sample;
use rand::Rng;
use specs::Entities;
use specs::Entity;
use specs::LazyUpdate;
use specs::ParJoin;
use specs::ReadStorage;
use specs::{Read, ReadExpect, System, WriteExpect};

/// A resource that indicates that the simulation should apply scattering
pub struct ApplyTwoBodyLossOption;
pub struct ApplyOneBodyLossOption;

#[derive(Clone)]
pub struct LossCoefficients {
    // Loss rate equal to 1 over one body lifetime (1/s)
    pub one_body_loss_rate: f64,

    // Two body loss rate coefficient (m^3/s)
    pub two_body_coefficient: f64,

    pub three_body_coefficient: f64,
}

impl Default for LossCoefficients {
    fn default() -> Self {
        LossCoefficients {
            one_body_loss_rate: 0.0,
            two_body_coefficient: 0.0,
            three_body_coefficient: 0.0,
        }
    }
}

impl PartitionCell {
    /// Perform 2-body loss within a box.
    fn two_body_loss(
        &mut self,
        partition_params: PartitionParameters,
        collision_params: CollisionParameters,
        two_body_coefficient: f64,
        dt: f64,
    ) -> Vec<Entity> {
        let atom_number = self.particle_number as f64 * collision_params.macroparticle;
        let density = atom_number / partition_params.box_width.powi(3);

        // two body loss rate: dN/dt = k_2 * n^2 * V = k_2 * n * N
        // So loss rate per particle is k_2 * n
        let mut num_losses = two_body_coefficient * density * dt * self.particle_number as f64;

        let mut rng = rand::thread_rng();
        let mut entities_to_be_destroyed = Vec::new();

        if num_losses > self.entities.len() as f64 {
            num_losses = self.entities.len() as f64;
        }
        if num_losses < 1.0 {
            if rng.gen::<f64>() < num_losses {
                let idx = sample(&mut rng, self.entities.len(), 1).index(0);
                entities_to_be_destroyed.push(self.entities[idx]);
            }
        } else if num_losses > 1.0 {
            let idx_rand = sample(&mut rng, self.entities.len(), num_losses.round() as usize);

            for idx in idx_rand {
                entities_to_be_destroyed.push(self.entities[idx]);
            }
        }

        entities_to_be_destroyed
    }
}

/// Performs collisions within the atom cloud using a spatially partitioned Monte-Carlo approach.
pub struct ApplyTwoBodyLossSystem;
impl<'a> System<'a> for ApplyTwoBodyLossSystem {
    type SystemData = (
        Option<Read<'a, ApplyTwoBodyLossOption>>,
        ReadExpect<'a, Timestep>,
        ReadExpect<'a, CollisionParameters>,
        ReadExpect<'a, PartitionParameters>,
        ReadExpect<'a, LossCoefficients>,
        WriteExpect<'a, DensityHashmap>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (loss_option, t, collision_params, partition_params, losses, mut hashmap,updater): Self::SystemData,
    ) {
        use rayon::prelude::*;

        match loss_option {
            None => (),
            Some(_) => {
                let cells: Vec<&mut PartitionCell> = hashmap.hashmap.values_mut().collect();
                cells.into_par_iter().for_each(|partition_cell| {
                    let entities_to_be_destroyed = partition_cell.two_body_loss(
                        partition_params.clone(),
                        collision_params.clone(),
                        losses.two_body_coefficient,
                        t.delta,
                    );

                    for e in entities_to_be_destroyed {
                        updater.insert(e, ToBeDestroyed);
                    }
                });
            }
        }
    }
}

/// Performs one body losses within the atom cloud using the spatial partition.
pub struct ApplyOneBodyLossSystem;
impl<'a> System<'a> for ApplyOneBodyLossSystem {
    type SystemData = (
        Option<Read<'a, ApplyOneBodyLossOption>>,
        ReadExpect<'a, Timestep>,
        ReadExpect<'a, LossCoefficients>,
        ReadStorage<'a, Atom>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (loss_option, t, losses, atoms, entities, updater): Self::SystemData) {
        use rayon::prelude::*;

        match loss_option {
            None => (),
            Some(_) => {
                // probability of atom being lost in a small time dt << lifetime
                let p_loss = t.delta * losses.one_body_loss_rate;

                (&entities, &atoms).par_join().for_each(|(entity, _atom)| {
                    let mut rng = rand::thread_rng();
                    if rng.gen::<f64>() < p_loss {
                        updater.insert(entity, ToBeDestroyed);
                    }
                });
            }
        }
    }
}

pub mod tests {

    #[allow(unused_imports)]
    use super::*;
    extern crate specs;
    #[allow(unused_imports)]
    use specs::prelude::*;
    extern crate nalgebra;
    #[allow(unused_imports)]
    use crate::atom::{Atom, Position};
    #[allow(unused_imports)]
    use crate::ecs;
    #[allow(unused_imports)]
    use crate::integrator::{Step, Timestep};
    #[allow(unused_imports)]
    use nalgebra::Vector3;

    #[test]
    fn test_one_body_loss() {
        let mut test_world = World::new();
        test_world.register::<Atom>();
        test_world.register::<Position>();
        test_world.register::<ToBeDestroyed>();
        test_world.insert(Timestep { delta: 1.0 });
        test_world.insert(ApplyOneBodyLossOption);
        test_world.insert(LossCoefficients {
            one_body_loss_rate: 1.0,
            two_body_coefficient: 0.0,
            three_body_coefficient: 0.0,
        });
        let atom = test_world
            .create_entity()
            .with(Atom)
            .with(Position::new())
            .build();

        let mut system = ApplyOneBodyLossSystem;

        system.run_now(&test_world);
        test_world.maintain();

        let tbd = test_world.read_storage::<ToBeDestroyed>();
        assert_eq!(tbd.get(atom).is_none(), false);
    }
}
