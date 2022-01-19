//! Creation of atoms in a controlled manner and realease into the simulation

pub mod emit;
pub mod gaussian;
pub mod mass;
pub mod oven;
pub mod precalc;
pub mod surface;
pub mod species;

use specs::prelude::*;

use rand;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::marker::PhantomData;

use self::species::AtomCreator;

pub struct VelocityCap {
    /// The maximum speed of an atom emitted by an atom source. See [Velocity](struct.Velocity.html) for units.
    pub value: f64,
}

/// Adds the systems required by `atom_sources` to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the atom_sources systems run.
pub fn add_systems_to_dispatch<T>(
    builder: &mut DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) where T : AtomCreator + 'static {
    builder.add(
        emit::EmitNumberPerFrameSystem,
        "emit_number_per_frame",
        deps,
    );
    builder.add(
        emit::EmitFixedRateSystem,
        "emit_fixed_rate",
        &["emit_number_per_frame"],
    );
    builder.add(
        precalc::PrecalculateForSpeciesSystem::<oven::Oven<T>> {
            marker: PhantomData,
        },
        "precalculated_oven",
        deps,
    );
    builder.add(
        precalc::PrecalculateForSpeciesSystem::<surface::SurfaceSource<T>> {
            marker: PhantomData,
        },
        "precalculated_surfaces",
        deps,
    );
    builder.add(
        gaussian::PrecalculateForGaussianSourceSystem::<T>::default(),
        "precalculate_gaussian",
        deps,
    );
    builder.add(
        oven::OvenCreateAtomsSystem::<T>::default(),
        "oven_create_atoms",
        &["emit_number_per_frame", "precalculated_oven"],
    );
    builder.add(
        surface::CreateAtomsOnSurfaceSystem::<T>::default(),
        "surface_create_atoms",
        &["emit_number_per_frame", "precalculated_surfaces"],
    );
    builder.add(
        gaussian::GaussianCreateAtomsSystem::<T>::default(),
        "gaussian_create_atoms",
        &["emit_number_per_frame", "precalculate_gaussian"],
    );
    builder.add(
        emit::EmitOnceSystem,
        "emit_once_system",
        &[
            "oven_create_atoms",
            "surface_create_atoms",
            "gaussian_create_atoms",
        ],
    );
}

/// Registers resources required by `atom_sources` to the ecs world.
pub fn register_components<T>(world: &mut World) where T : AtomCreator + 'static {
    world.register::<oven::Oven<T>>();
    world.register::<mass::MassDistribution>();
    world.register::<emit::EmitFixedRate>();
    world.register::<emit::EmitNumberPerFrame>();
    world.register::<emit::EmitOnce>();
    world.register::<emit::AtomNumberToEmit>();
    world.register::<surface::SurfaceSource<T>>();
    world.register::<gaussian::GaussianVelocityDistributionSource<T>>();
    world.register::<gaussian::GaussianVelocityDistributionSourceDefinition<T>>();
}

/// A simple probability distribution which uses weighted indices to retrieve values.
pub struct WeightedProbabilityDistribution {
    values: Vec<f64>,
    weighted_index: WeightedIndex<f64>,
}
impl WeightedProbabilityDistribution {
    pub fn new(values: Vec<f64>, weights: Vec<f64>) -> Self {
        WeightedProbabilityDistribution {
            values,
            weighted_index: WeightedIndex::new(&weights).unwrap(),
        }
    }
}
impl Distribution<f64> for WeightedProbabilityDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let index = self.weighted_index.sample(rng);
        self.values[index]
    }
}
