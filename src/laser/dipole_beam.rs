extern crate specs;
use crate::constant;
use serde::{Deserialize, Serialize};
use specs::{
    Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage,
};

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

/// An index that uniquely identifies this dipole light in the interaction list for each atom.
/// The index value corresponds to the position of each dipole light in the per-atom interaction list array.
///
/// Default `DipoleLightIndex`s are created with `initiated: false`.
/// Once the index is set, initiated is set to true.
/// This is used to detect if all lasers in the simulation are correctly indexed, in case new lasers are added.
#[derive(Clone, Copy)]
pub struct DipoleLightIndex {
    pub index: usize,
    pub initiated: bool,
}
impl Component for DipoleLightIndex {
    type Storage = HashMapStorage<Self>;
}
impl Default for DipoleLightIndex {
    fn default() -> Self {
        DipoleLightIndex {
            index: 0,
            initiated: false,
        }
    }
}

/// Assigns unique indices to dipole light entities.
///
/// The indices are used to uniquely identify each dipole light when populating the interaction list.
pub struct IndexDipoleLightsSystem;
impl<'a> System<'a> for IndexDipoleLightsSystem {
    type SystemData = (
        ReadStorage<'a, DipoleLight>,
        WriteStorage<'a, DipoleLightIndex>,
    );

    fn run(&mut self, (dipole_light, mut indices): Self::SystemData) {
        let mut iter = 0;
        let mut need_to_assign_indices = false;

        for (_, index) in (&dipole_light, &indices).join() {
            if index.initiated == false {
                need_to_assign_indices = true;
            }
        }
        if need_to_assign_indices {
            for (_, mut index) in (&dipole_light, &mut indices).join() {
                index.index = iter;
                index.initiated = true;
                iter = iter + 1;
            }
        }
    }
}

/// A system that attaches `DipoleLightIndex` components to entities which have `DipoleLight` but no index.
pub struct AttachIndexToDipoleLightSystem;
impl<'a> System<'a> for AttachIndexToDipoleLightSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, DipoleLightIndex>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, dipole_light, dipole_light_index, updater): Self::SystemData) {
        for (ent, _, _) in (&ent, &dipole_light, !&dipole_light_index).join() {
            updater.insert(ent, DipoleLightIndex::default());
        }
    }
}
