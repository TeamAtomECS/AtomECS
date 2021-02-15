//! Shift in an atom's transition frequency due to a magnetic field (zeeman effect)

use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage,
};

use super::MagneticFieldSampler;
use crate::atom::AtomicTransition;
use crate::constant::HBAR;
use crate::initiate::NewlyCreated;

/// Represents the (angular) Zeemanshift of the atom depending on the magnetic field it experiences
#[derive(Clone)]
pub struct ZeemanShiftSampler {
    /// Zeemanshift for sigma plus transition in rad/s
    pub sigma_plus: f64,
    /// Zeemanshift for sigma minus transition in rad/s
    pub sigma_minus: f64,
    /// Zeemanshift for pi transition in rad/s
    pub sigma_pi: f64,
}

impl Default for ZeemanShiftSampler {
    fn default() -> Self {
        ZeemanShiftSampler {
            /// Zeemanshift for sigma plus transition in rad/s
            sigma_plus: f64::NAN,
            /// Zeemanshift for sigma minus transition in rad/s
            sigma_minus: f64::NAN,
            /// Zeemanshift for pi transition in rad/s
            sigma_pi: f64::NAN,
        }
    }
}

impl Component for ZeemanShiftSampler {
    type Storage = VecStorage<Self>;
}

/// Attaches the ZeemanShifSampler component to newly created atoms.
pub struct AttachZeemanShiftSamplersToNewlyCreatedAtomsSystem;

impl<'a> System<'a> for AttachZeemanShiftSamplersToNewlyCreatedAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, NewlyCreated>,
        ReadStorage<'a, AtomicTransition>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (ent, newly_created, atomic_transition, updater): Self::SystemData) {
        for (ent, _nc, _at) in (&ent, &newly_created, &atomic_transition).join() {
            updater.insert(ent, ZeemanShiftSampler::default());
        }
    }
}

/// Calculates the Zeeman shift for each atom in each cooling beam.
pub struct CalculateZeemanShiftSystem;
impl<'a> System<'a> for CalculateZeemanShiftSystem {
    type SystemData = (
        WriteStorage<'a, ZeemanShiftSampler>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, AtomicTransition>,
    );

    fn run(
        &mut self,
        (mut zeeman_sampler, magnetic_field_sampler, atomic_transition): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (
            &mut zeeman_sampler,
            &magnetic_field_sampler,
            &atomic_transition,
        )
            .par_join()
            .for_each(|(zeeman, magnetic_field, atom_info)| {
                zeeman.sigma_plus = atom_info.mup / HBAR * magnetic_field.magnitude;
                zeeman.sigma_minus = atom_info.mum / HBAR * magnetic_field.magnitude;
                zeeman.sigma_pi = atom_info.muz / HBAR * magnetic_field.magnitude;
            });
    }
}
