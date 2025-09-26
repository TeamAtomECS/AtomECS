//! Creation of atoms in a controlled manner and realease into the simulation

pub mod emit;
pub mod gaussian;
pub mod mass;
pub mod oven;
pub mod precalc;
pub mod surface;
pub mod species;

use bevy::prelude::*;

use rand;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::Rng;
use std::marker::PhantomData;

use self::species::AtomCreator;

#[derive(Resource)]
pub struct VelocityCap {
    /// The maximum speed of an atom emitted by an atom source. See [Velocity](struct.Velocity.html) for units.
    pub value: f64,
}

impl Default for VelocityCap {
    fn default() -> Self {
        VelocityCap { value: std::f64::MAX } // Default to no cap on velocity
    }
}

/// This plugin implements the creation of atoms of a given species from sources such as ovens or vacuum chambers.
/// 
/// See also [crate::atom_sources].
/// 
/// # Generic Arguments
/// 
/// * `T`: The atom species to create, which must implement the `AtomCreator` trait.
#[derive(Default)]
pub struct AtomSourcePlugin<T>(PhantomData<T>) where T : AtomCreator;

impl<T> Plugin for AtomSourcePlugin<T> where T : AtomCreator + 'static {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                emit::emit_number_per_frame_system,
                emit::emit_fixed_rate_system.after(emit::emit_number_per_frame_system),
                precalc::precalculate_for_species_system::<oven::Oven<T>>,
                precalc::precalculate_for_species_system::<surface::SurfaceSource<T>>,
                gaussian::precalculate_for_gaussian_source_system::<T>,
                oven::oven_create_atoms_system::<T>
                    .after(emit::emit_number_per_frame_system)
                    .after(precalc::precalculate_for_species_system::<oven::Oven<T>>),
                surface::create_atoms_on_surface_system::<T>
                    .after(emit::emit_number_per_frame_system)
                    .after(precalc::precalculate_for_species_system::<surface::SurfaceSource<T>>),
                gaussian::gaussian_create_atoms_system::<T>
                    .after(emit::emit_number_per_frame_system)
                    .after(gaussian::precalculate_for_gaussian_source_system::<T>),
                emit::emit_once_system
                    .after(oven::oven_create_atoms_system::<T>)
                    .after(surface::create_atoms_on_surface_system::<T>)
                    .after(gaussian::gaussian_create_atoms_system::<T>)
            ),
        );
    }
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
