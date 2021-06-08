extern crate rayon;
extern crate specs;
use crate::atom::{Atom, AtomicTransition, Kind};
use crate::destructor::ToBeDestroyed;
use crate::dipole::atom::AtomicDipoleTransition;
use crate::laser::cooling::CoolingLight;
use crate::laser::dipole_beam::DipoleLight;
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System};
extern crate nalgebra;

/// System that globally deletes all `CoolingLight` entities that do not have
/// a `DipoleLight`. This could be used to speed up the simulation after a completed MOT power-ramp
/// and transition into a pure dipole trap.
pub struct DisableMOTBeamsSystem;

impl<'a> System<'a> for DisableMOTBeamsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, DipoleLight>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ents, cooling, dipole, updater): Self::SystemData) {
        for (entity, _cooling, _dipole) in (&ents, &cooling, !&dipole).join() {
            updater.insert(entity, ToBeDestroyed);
        }
    }
}

pub struct DisableBlueMOTBeamsSystem;

impl<'a> System<'a> for DisableBlueMOTBeamsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, DipoleLight>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ents, cooling, dipole, updater): Self::SystemData) {
        for (entity, cooling, _dipole) in (&ents, &cooling, !&dipole).join() {
            if cooling.wavelength < 500.0e-9 {
                updater.insert(entity, ToBeDestroyed);
            }
        }
    }
}
pub struct AttachNewAtomicTransitionToAtomsSystem;

impl<'a> System<'a> for AttachNewAtomicTransitionToAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomicTransition>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ents, atom, atomic_transition, updater): Self::SystemData) {
        for (entity, _atom, atominfo) in (&ents, &atom, &atomic_transition).join() {
            updater.insert(
                entity,
                match atominfo.kind {
                    Kind::Rubidium => AtomicTransition::rubidium(),
                    Kind::Strontium => AtomicTransition::strontium_red(),
                    Kind::StrontiumRed => AtomicTransition::strontium_red(),
                    Kind::Erbium => AtomicTransition::erbium(),
                    Kind::Erbium401 => AtomicTransition::erbium_401(),
                },
            );
        }
    }
}
/// Depending on the kind of `AtomicTransition`, this system associates the atom entities
/// with an `AtomicDipoleTransition` so they are can be processed by the `ApplyDipoleForceSystem.`
pub struct AttachAtomicDipoleTransitionToAtomsSystem;

impl<'a> System<'a> for AttachAtomicDipoleTransitionToAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, AtomicDipoleTransition>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (ents, atom, atomic_transition, atomic_dipole_transition, updater): Self::SystemData,
    ) {
        for (entity, _atom, atominfo, _atomdipole_info) in
            (&ents, &atom, &atomic_transition, !&atomic_dipole_transition).join()
        {
            updater.insert(
                entity,
                match atominfo.kind {
                    Kind::Rubidium => AtomicDipoleTransition::rubidium(),
                    Kind::Strontium => AtomicDipoleTransition::strontium(),
                    Kind::StrontiumRed => AtomicDipoleTransition::strontium(),
                    Kind::Erbium => AtomicDipoleTransition::erbium(),
                    Kind::Erbium401 => AtomicDipoleTransition::erbium_401(),
                },
            );
        }
    }
}
