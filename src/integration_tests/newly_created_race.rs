//! Regression test for the `NewlyCreated` component-attachment race.
//!
//! Systems that attach per-atom samplers to `NewlyCreated` atoms must run before
//! `deflag_new_atoms` removes the marker. If one runs after, its `With<NewlyCreated>` query does
//! not match and the component is silently never attached, leaving atoms without, e.g.,
//! `MagneticFieldSampler` or `ZeemanShiftSampler`, and hence feeling no force.
//!
//! Registering every attach system in [`crate::initiate::InitiateSet::AttachToNewlyCreated`] is
//! what guarantees the ordering. This test builds the full plugin stack under several schedule
//! perturbations (extra `Commands` systems reshuffle the topological sort) and asserts that every
//! newly created atom receives both samplers in each case.

#[cfg(test)]
mod tests {
    use crate::atom::{Atom, Position};
    use crate::atom_sources::{AtomSourcePlugin, VelocityCap};
    use crate::initiate::NewlyCreated;
    use crate::laser::LaserPlugin;
    use crate::laser_cooling::zeeman::ZeemanShiftSampler;
    use crate::laser_cooling::LaserCoolingPlugin;
    use crate::magnetic::MagneticFieldSampler;
    use crate::simulation::SimulationBuilder;
    use crate::species::{Strontium88, Strontium88_461};
    use bevy::prelude::*;

    const N: usize = 6;
    const ATOM_COUNT: usize = 8;

    // Distinct trivial systems, each with a deferred (`Commands`) parameter, used to perturb
    // the schedule's topological sort (mimicking systems an output/diagnostics plugin adds).
    fn perturb_a(_c: Commands, _q: Query<Entity, With<Atom>>) {}
    fn perturb_b(_c: Commands, _q: Query<Entity, With<Position>>) {}
    fn perturb_c(_c: Commands) {}

    /// Builds the full stack, spawns `ATOM_COUNT` `NewlyCreated` atoms, runs one update, and
    /// returns `(atoms with MagneticFieldSampler, atoms with ZeemanShiftSampler)`.
    fn count_attached(num_extra: usize) -> (usize, usize) {
        let mut sim = SimulationBuilder::default().build();
        sim.add_plugins(LaserPlugin::<N>);
        sim.add_plugins(LaserCoolingPlugin::<Strontium88_461, N>::default());
        // `AtomSourcePlugin` brings in `emit_once_system`, which forces a mid-Update sync point.
        sim.add_plugins(AtomSourcePlugin::<Strontium88>::default());
        // Required by the atom-source create systems added by `AtomSourcePlugin`.
        sim.insert_resource(VelocityCap { value: 200.0 });

        if num_extra > 0 {
            sim.add_systems(Update, perturb_a);
        }
        if num_extra > 1 {
            sim.add_systems(Update, perturb_b);
        }
        if num_extra > 2 {
            sim.add_systems(Update, perturb_c);
        }

        let atoms: Vec<Entity> = (0..ATOM_COUNT)
            .map(|_| {
                sim.world_mut()
                    .spawn((Atom, Position::default(), Strontium88_461, NewlyCreated))
                    .id()
            })
            .collect();

        sim.update();

        let with_field = atoms
            .iter()
            .filter(|e| sim.world().get::<MagneticFieldSampler>(**e).is_some())
            .count();
        let with_zeeman = atoms
            .iter()
            .filter(|e| sim.world().get::<ZeemanShiftSampler<Strontium88_461>>(**e).is_some())
            .count();
        (with_field, with_zeeman)
    }

    /// Every newly created atom must receive both `MagneticFieldSampler` and
    /// `ZeemanShiftSampler`, regardless of how extra systems perturb the schedule.
    #[test]
    fn newly_created_atoms_receive_all_samplers() {
        for num_extra in 0..=3 {
            let (with_field, with_zeeman) = count_attached(num_extra);
            assert_eq!(
                with_field, ATOM_COUNT,
                "MagneticFieldSampler missing on some atoms (num_extra={num_extra}): {with_field}/{ATOM_COUNT}"
            );
            assert_eq!(
                with_zeeman, ATOM_COUNT,
                "ZeemanShiftSampler missing on some atoms (num_extra={num_extra}): {with_zeeman}/{ATOM_COUNT}"
            );
        }
    }
}
