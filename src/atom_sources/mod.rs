//! Creation of atoms in a controlled manner and realease into the simulation

pub mod central_creator;
pub mod emit;
pub mod gaussian;
pub mod mass;
pub mod oven;
pub mod precalc;
pub mod surface;

use specs::{DispatcherBuilder, World};

extern crate rand;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::marker::PhantomData;

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
pub fn add_systems_to_dispatch(
    builder: DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) -> DispatcherBuilder<'static, 'static> {
    builder
        .with(
            emit::EmitNumberPerFrameSystem,
            "emit_number_per_frame",
            deps,
        )
        .with(
            emit::EmitFixedRateSystem,
            "emit_fixed_rate",
            &["emit_number_per_frame"],
        )
        .with(
            precalc::PrecalculateForSpeciesSystem::<oven::Oven> {
                marker: PhantomData,
            },
            "precalculated_oven",
            deps,
        )
        .with(
            precalc::PrecalculateForSpeciesSystem::<surface::SurfaceSource> {
                marker: PhantomData,
            },
            "precalculated_surfaces",
            deps,
        )
        .with(
            gaussian::PrecalculateForGaussianSourceSystem,
            "precalculate_gaussian",
            deps,
        )
        .with(
            oven::OvenCreateAtomsSystem,
            "oven_create_atoms",
            &["emit_number_per_frame", "precalculated_oven"],
        )
        .with(
            surface::CreateAtomsOnSurfaceSystem,
            "surface_create_atoms",
            &["emit_number_per_frame", "precalculated_surfaces"],
        )
        .with(
            gaussian::GaussianCreateAtomsSystem,
            "gaussian_create_atoms",
            &["emit_number_per_frame", "precalculate_gaussian"],
        )
        .with(
            emit::EmitOnceSystem,
            "emit_once_system",
            &[
                "oven_create_atoms",
                "surface_create_atoms",
                "gaussian_create_atoms",
            ],
        )
        .with(
            central_creator::CentralCreatorCreateAtomsSystem,
            "central_create_system",
            &[],
        )
}

/// Registers resources required by `atom_sources` to the ecs world.
pub fn register_components(world: &mut World) {
    world.register::<oven::Oven>();
    world.register::<mass::MassDistribution>();
    world.register::<emit::EmitFixedRate>();
    world.register::<emit::EmitNumberPerFrame>();
    world.register::<emit::EmitOnce>();
    world.register::<emit::AtomNumberToEmit>();
    world.register::<surface::SurfaceSource>();
    world.register::<gaussian::GaussianVelocityDistributionSource>();
    world.register::<gaussian::GaussianVelocityDistributionSourceDefinition>();
    world.register::<central_creator::CentralCreator>();
}

/// A simple probability distribution which uses weighted indices to retrieve values.
struct WeightedProbabilityDistribution {
    values: Vec<f64>,
    weighted_index: WeightedIndex<f64>,
}
impl WeightedProbabilityDistribution {
    pub fn new(values: Vec<f64>, weights: Vec<f64>) -> Self {
        WeightedProbabilityDistribution {
            values: values,
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
