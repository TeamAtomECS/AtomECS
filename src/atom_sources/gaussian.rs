//! Atom sources with gaussian velocity distributions.

use std::marker::PhantomData;

use super::{WeightedProbabilityDistribution, species::AtomCreator};
use crate::atom::*;
use crate::atom_sources::emit::AtomNumberToEmit;
use crate::constant::EXP;
use crate::initiate::*;
use nalgebra::Vector3;

use rand;
use rand::distr::Distribution;
use rand::Rng;

use bevy::prelude::*;

/// Component which defines a gaussian velocity distribution source.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct GaussianVelocityDistributionSourceDefinition<T> where T : AtomCreator {
    pub mean: Vector3<f64>,
    pub std: Vector3<f64>,
    phantom: PhantomData<T>
}

/// Component which contains precalculated velocity distributions for a gaussian source.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct GaussianVelocityDistributionSource<T> where T : AtomCreator {
    vx_distribution: WeightedProbabilityDistribution,
    vy_distribution: WeightedProbabilityDistribution,
    vz_distribution: WeightedProbabilityDistribution,
    phantom: PhantomData<T>
}

impl<T> GaussianVelocityDistributionSource<T> where T : AtomCreator {
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

pub fn precalculate_for_gaussian_source_system<T>(
    mut query: Query<(Entity, &GaussianVelocityDistributionSourceDefinition<T>,), Without<GaussianVelocityDistributionSource<T>>>,
    mut commands: Commands,
) where T: AtomCreator + 'static {
    // println!("precalculate for gaussian source system");
    let mut precalculated_data = Vec::<(Entity, GaussianVelocityDistributionSource<T>)>::new();
    for (entity, definition) in query.iter_mut() {
        let source = GaussianVelocityDistributionSource {
            vx_distribution: create_gaussian_velocity_distribution(definition.mean[0], definition.std[0]),
            vy_distribution: create_gaussian_velocity_distribution(definition.mean[1], definition.std[1]),
            vz_distribution: create_gaussian_velocity_distribution(definition.mean[2], definition.std[2]),
            phantom: PhantomData
        };
        precalculated_data.push((entity, source));
        println!("Precalculated velocity and mass distributions for a Gaussian source.");
    }

    for (entity, precalculated) in precalculated_data {
        commands.entity(entity).insert(precalculated); // Insert the precalculated data
    }
}

pub fn gaussian_create_atoms_system<T>(
    mut query: Query<
        (&GaussianVelocityDistributionSource<T>,
        &AtomNumberToEmit,
        &Position,
        &Mass,)
        >,
    mut commands: Commands,
) where T: AtomCreator + 'static {
    let mut rng = rand::rng();
    for (source, number_to_emit, source_position, mass) in query.iter_mut() {
        for _i in 0..number_to_emit.number {
            let new_vel = source.get_random_velocity(&mut rng);
            let new_atom = commands
                .spawn((
                    Velocity { vel: new_vel },
                    source_position.clone(),
                    Force::default(),
                    mass.clone(),
                    Atom,
                    InitialVelocity { vel: new_vel },
                    NewlyCreated,
                ))
                .id();
            T::mutate(&mut commands, new_atom);
        }
    }
}
