//! Atom sources with gaussian velocity distributions.

use super::WeightedProbabilityDistribution;
use crate::atom::*;
use crate::atom_sources::emit::AtomNumberToEmit;
use crate::constant::EXP;
use crate::initiate::*;
use nalgebra::Vector3;

use rand;
use rand::distributions::Distribution;
use rand::Rng;

use specs::{
    Component, Entities, Entity, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System,
    WriteStorage,
};

pub struct GaussianVelocityDistributionSourceDefinition {
    pub mean: Vector3<f64>,
    pub std: Vector3<f64>,
}
impl Component for GaussianVelocityDistributionSourceDefinition {
    type Storage = HashMapStorage<Self>;
}

pub struct GaussianVelocityDistributionSource {
    vx_distribution: WeightedProbabilityDistribution,
    vy_distribution: WeightedProbabilityDistribution,
    vz_distribution: WeightedProbabilityDistribution,
}
impl Component for GaussianVelocityDistributionSource {
    type Storage = HashMapStorage<Self>;
}
impl GaussianVelocityDistributionSource {
    fn get_random_velocity<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3<f64> {
        Vector3::new(
            self.vx_distribution.sample(rng),
            self.vy_distribution.sample(rng),
            self.vz_distribution.sample(rng),
        )
    }
}

/// Creates and precalculates a [WeightedProbabilityDistribution](struct.WeightedProbabilityDistribution.html)
/// which can be used to sample values of velocity, based on given mean/std.
///
/// # Arguments
///
/// `mean`: The mean velocity, in m/s
///
/// `std`: The std of velocity, in m/s
pub fn create_gaussian_velocity_distribution(
    mean: f64,
    std: f64,
) -> WeightedProbabilityDistribution {
    // tuple list of (velocity, weight)
    let mut velocities = Vec::<f64>::new();
    let mut weights = Vec::<f64>::new();

    // precalculate the discretized distribution.
    let n = 1000;
    for i in -n..n {
        let v = (i as f64) / (n as f64) * 5.0 * std;
        let weight = EXP.powf(-(v / std).powf(2.0) / 2.0);
        velocities.push(v + mean);
        weights.push(weight);
    }

    WeightedProbabilityDistribution::new(velocities, weights)
}

/// Precalculates the probability distributions for
/// [GaussianVelocityDistributionSourceDefinition](struct.GaussianVelocityDistributionSourceDefinition.html) and
/// stores the result in a [GaussianVelocityDistributionSource](struct.GaussianVelocityDistributionSource.html) component.
pub struct PrecalculateForGaussianSourceSystem;
impl<'a> System<'a> for PrecalculateForGaussianSourceSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, GaussianVelocityDistributionSourceDefinition>,
        WriteStorage<'a, GaussianVelocityDistributionSource>,
    );

    fn run(&mut self, (entities, definitions, mut calculated): Self::SystemData) {
        let mut precalculated_data = Vec::<(Entity, GaussianVelocityDistributionSource)>::new();
        for (entity, definition, _) in (&entities, &definitions, !&calculated).join() {
            let source = GaussianVelocityDistributionSource {
                vx_distribution: create_gaussian_velocity_distribution(
                    definition.mean[0],
                    definition.std[0],
                ),
                vy_distribution: create_gaussian_velocity_distribution(
                    definition.mean[1],
                    definition.std[1],
                ),
                vz_distribution: create_gaussian_velocity_distribution(
                    definition.mean[2],
                    definition.std[2],
                ),
            };
            precalculated_data.push((entity, source));
            println!("Precalculated velocity and mass distributions for a gaussian source.");
        }

        for (entity, precalculated) in precalculated_data {
            calculated
                .insert(entity, precalculated)
                .expect("Could not add precalculated gaussian source.");
        }
    }
}

pub struct GaussianCreateAtomsSystem;

impl<'a> System<'a> for GaussianCreateAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, GaussianVelocityDistributionSource>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, AtomNumberToEmit>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Mass>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (entities, sources, atom_infos, numbers_to_emits, positions, masses, updater): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        for (source, atom, number_to_emit, source_position, mass) in (
            &sources,
            &atom_infos,
            &numbers_to_emits,
            &positions,
            &masses,
        )
            .join()
        {
            for _i in 0..number_to_emit.number {
                let new_atom = entities.create();
                let new_vel = source.get_random_velocity(&mut rng);
                updater.insert(
                    new_atom,
                    Velocity {
                        vel: new_vel,
                    },
                );
                updater.insert(new_atom, source_position.clone());
                updater.insert(new_atom, Force::new());
                updater.insert(new_atom, mass.clone());
                updater.insert(new_atom, *atom);
                updater.insert(new_atom, Atom);
                updater.insert(new_atom, InitialVelocity { vel: new_vel });
                updater.insert(new_atom, NewlyCreated);
            }
        }
    }
}
