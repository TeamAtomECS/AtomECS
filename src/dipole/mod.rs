//! A module that implements systems and components for dipole trapping in AtomECS.

use specs::DispatcherBuilder;

use crate::constant;
use crate::laser::index::LaserIndex;

use serde::{Deserialize, Serialize};
use specs::prelude::*;

pub mod atom;
pub mod force;

/// A component marking the entity as laser beam for dipole forces and
/// holding properties of the light
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct DipoleLight {
    ///wavelength of the laser light in SI units of m.
    pub wavelength: f64,
}

impl DipoleLight {
    /// Frequency of the dipole light in units of Hz
    pub fn frequency(&self) -> f64 {
        constant::C / self.wavelength
    }

    /// Wavenumber of the dipole light, in units of 2pi/m
    pub fn wavenumber(&self) -> f64 {
        2.0 * constant::PI / self.wavelength
    }
}

impl Component for DipoleLight {
    type Storage = HashMapStorage<Self>;
}

/// A system that attaches `DipoleLightIndex` components to entities which have `DipoleLight` but no index.
pub struct AttachIndexToDipoleLightSystem;
impl<'a> System<'a> for AttachIndexToDipoleLightSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, LaserIndex>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, dipole_light, indices, updater): Self::SystemData) {
        for (ent, _, _) in (&ent, &dipole_light, !&indices).join() {
            updater.insert(ent, LaserIndex::default());
        }
    }
}

/// Adds the systems required by the module to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the systems run.
pub fn add_systems_to_dispatch(builder: &mut DispatcherBuilder<'static, 'static>, deps: &[&str]) {
    builder.add(
        force::ApplyDipoleForceSystem,
        "apply_dipole_force",
        &["sample_intensity_gradient"],
    );
    builder.add(
        crate::dipole::AttachIndexToDipoleLightSystem,
        "attach_dipole_index",
        deps,
    );
}

pub fn register_components(world: &mut World) {
    world.register::<DipoleLight>();
}
