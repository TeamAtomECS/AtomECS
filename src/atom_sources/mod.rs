pub mod emit;
pub mod mass;
pub mod oven;

use specs::{DispatcherBuilder, World};

extern crate rand;
use rand::Rng;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;

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
            oven::PrecalculateForSpeciesSystem, 
            "precalculated_oven",
            deps,
        )
        .with(oven::OvenCreateAtomsSystem, "", &["emit_number_per_frame", "precalculated_oven"])
}

/// Registers resources required by `atom_sources` to the ecs world.
pub fn register_components(world: &mut World) {
    world.register::<oven::Oven>();
    world.register::<mass::MassDistribution>();
    world.register::<emit::EmitFixedRate>();
    world.register::<emit::EmitNumberPerFrame>();
    world.register::<emit::AtomNumberToEmit>();
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
