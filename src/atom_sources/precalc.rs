//! Utilities for precalculating quantities such as mass and velocity distributions.

use super::mass::MassDistribution;
use super::WeightedProbabilityDistribution;
use crate::constant::{AMU, BOLTZCONST, EXP};

use rand;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::marker::PhantomData;

use specs::{Component, Entities, Entity, HashMapStorage, Join, ReadStorage, System, WriteStorage};

/// Creates and precalculates a [WeightedProbabilityDistribution](struct.WeightedProbabilityDistribution.html)
/// which can be used to sample values of velocity, based on the effusive Maxwell-Boltzmann distribution.
///
/// # Arguments
///
/// `temperature`: The temperature of the oven, in units of Kelvin.
///
/// `mass`: The mass of the particle, in SI units of kg.
fn create_v_distribution(
    temperature: f64,
    mass: f64,
    power: f64,
) -> WeightedProbabilityDistribution {
    let max_velocity = 7.0 * (2.0 * BOLTZCONST * temperature / mass).powf(0.5);

    // tuple list of (velocity, weight)
    let mut velocities = Vec::<f64>::new();
    let mut weights = Vec::<f64>::new();

    // precalculate the discretized distribution.
    let n = 2000;
    for i in 0..n {
        let v = (i as f64 + 0.5) / (n as f64 + 1.0) * max_velocity;
        let weight = probability_v(temperature, mass, v, power);
        velocities.push(v);
        weights.push(weight);
    }

    WeightedProbabilityDistribution::new(velocities, weights)
}

/// The probability distribution `p(v)` that a given `mass` has a velocity magnitude `v`.
/// This distrubiton has p(v) \propto v^3 exp(-v)
///
/// # Arguments
///
/// `temperature`: temperature of the gas, in Kelvin.
///
/// `mass`: particle mass, in SI units of kg.
///
/// `v`: velocity magnitude, in SI units of m/s.
///
/// See _Atomic and Molecular Beam Methods_, Scoles, p85
pub fn probability_v(temperature: f64, mass: f64, v: f64, power: f64) -> f64 {
    let norm_v = v / (2.0 * BOLTZCONST * temperature / mass).powf(0.5); // (4.2) and (4.4)
    2.0 * norm_v.powf(power) * EXP.powf(-norm_v.powf(2.0))
}

/// Holds any precalculated information required to generate atoms of the given species.
pub struct Species {
    /// Mass of the species, in atomic mass units
    mass: f64,
    /// Distribution that can be used to generate random velocity magnitudes `v`.
    v_distribution: WeightedProbabilityDistribution,
}
impl Species {
    fn create(mass: f64, temperature: f64, power: f64) -> Self {
        Species {
            mass,
            v_distribution: create_v_distribution(temperature, mass * AMU, power),
        }
    }
}

/// Holds all precalculated information required for generating atoms on a per-species basis.
pub struct PrecalculatedSpeciesInformation {
    /// All species that can be generated
    species: Vec<Species>,
    /// weighted distribution holding the chance to create each species.
    distribution: WeightedIndex<f64>,
}
impl PrecalculatedSpeciesInformation {
    /// Gets a random mass and velocity from the precalculated distributions.
    ///
    /// The tuple returned is of the form (mass, velocity). The mass is measured
    /// in atomic mass units, and the velocity magnitude is measured in m/s.
    pub fn generate_random_mass_v<R: Rng + ?Sized>(&self, rng: &mut R) -> (f64, f64) {
        let i = self.distribution.sample(rng);
        let species = &self.species[i];
        (species.mass, species.v_distribution.sample(rng))
    }

    fn create(temperature: f64, mass_distribution: &MassDistribution, power: f64) -> Self {
        let mut species = Vec::<Species>::new();
        let mut ratios = Vec::<f64>::new();
        for mr in &mass_distribution.distribution {
            ratios.push(mr.ratio);
            species.push(Species::create(mr.mass, temperature, power));
        }
        PrecalculatedSpeciesInformation {
            species,
            distribution: WeightedIndex::new(&ratios).unwrap(),
        }
    }
}
impl Component for PrecalculatedSpeciesInformation {
    type Storage = HashMapStorage<Self>;
}

pub trait MaxwellBoltzmannSource {
    fn get_temperature(&self) -> f64;
    fn get_v_dist_power(&self) -> f64;
}

/// Precalculates different distributions used by the Oven systems.
///
/// This system removes the [MassDistribution](struct.MassDistribution.html) component from
/// the oven and replaces it with a [PrecalculatedForSpeciesSystem] that contains all
/// precalculated information required to generate atoms from the distribution.
#[derive(Default)]
pub struct PrecalculateForSpeciesSystem<T: MaxwellBoltzmannSource> {
    pub marker: PhantomData<T>,
}

impl<'a, T> System<'a> for PrecalculateForSpeciesSystem<T>
where
    T: MaxwellBoltzmannSource + Component,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, T>,
        WriteStorage<'a, MassDistribution>,
        WriteStorage<'a, PrecalculatedSpeciesInformation>,
    );

    fn run(&mut self, (entities, sources, mut mass_distributions, mut precalcs): Self::SystemData) {
        // Precalculate for ovens which do not currently have precalculated information.
        let mut precalculated_data = Vec::<(Entity, PrecalculatedSpeciesInformation)>::new();
        for (entity, source, mass_dist, _) in
            (&entities, &sources, &mass_distributions, !&precalcs).join()
        {
            let precalculated = PrecalculatedSpeciesInformation::create(
                source.get_temperature(),
                mass_dist,
                source.get_v_dist_power(),
            );
            //mass_distributions.remove(entity);
            //precalcs.insert(entity, precalculated);
            precalculated_data.push((entity, precalculated));
            println!("Precalculated velocity and mass distributions for an oven.");
        }

        for (entity, precalculated) in precalculated_data {
            mass_distributions.remove(entity);
            precalcs
                .insert(entity, precalculated)
                .expect("Could not add precalculated data to oven.");
        }
    }
}
