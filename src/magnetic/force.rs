//! Magnetic dipole force applied to an atom in an external magnetic field

use super::MagneticFieldSampler;
use crate::constant::HBAR;
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage,
};

/// Component that represents the magnetic dipole moment of an atom.
pub struct MagneticDipole {
    /// Zeeman state mF
    pub mF: f64,
    /// Lande g factor
    pub gF: f64,
}

impl Component for MagneticDipole {
    type Storage = VecStorage<Self>;
}

pub struct ApplyMagneticForceSystem;
impl<'a> System<'a> for ApplyMagneticForceSystem {
    type SystemData = (
        WriteStorage<'a, Force>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, MagneticDipole>,
    );

    fn run(&mut self, (mut forces, samplers, dipoles): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut forces, &samplers, &dipoles)
            .par_join()
            .foreach(|mut force, sampler, dipole| {
                force.force =
                    force.force + dipole.mF * dipole.gF * constant::BOHRMAG * sampler.gradient;
            });
    }
}
