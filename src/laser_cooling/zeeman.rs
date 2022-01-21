//! Shift in an atom's transition frequency due to a magnetic field (zeeman effect)
extern crate serde;
use std::marker::PhantomData;

use crate::magnetic::MagneticFieldSampler;
use crate::constant::HBAR;
use crate::initiate::NewlyCreated;
use serde::Serialize;
use specs::prelude::*;

use super::transition::TransitionComponent;

/// Represents the (angular) Zeemanshift of the atom depending on the magnetic field it experiences
#[derive(Clone, Copy, Serialize)]
pub struct ZeemanShiftSampler<T> where T : TransitionComponent {
    /// Zeemanshift for sigma plus transition in rad/s
    pub sigma_plus: f64,
    /// Zeemanshift for sigma minus transition in rad/s
    pub sigma_minus: f64,
    /// Zeemanshift for pi transition in rad/s
    pub sigma_pi: f64,
    phantom: PhantomData<T>
}

impl<T> Default for ZeemanShiftSampler<T> where T : TransitionComponent {
    fn default() -> Self {
        ZeemanShiftSampler::<T> {
            /// Zeemanshift for sigma plus transition in rad/s
            sigma_plus: f64::NAN,
            /// Zeemanshift for sigma minus transition in rad/s
            sigma_minus: f64::NAN,
            /// Zeemanshift for pi transition in rad/s
            sigma_pi: f64::NAN,
            phantom: PhantomData
        }
    }
}

impl<T> Component for ZeemanShiftSampler<T> where T : TransitionComponent + 'static {
    type Storage = VecStorage<Self>;
}

/// Attaches the ZeemanShifSampler component to newly created atoms.
#[derive(Default)]
pub struct AttachZeemanShiftSamplersToNewlyCreatedAtomsSystem<T>(PhantomData<T>) where T : TransitionComponent;

impl<'a, T> System<'a> for AttachZeemanShiftSamplersToNewlyCreatedAtomsSystem<T> where T : TransitionComponent {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, NewlyCreated>,
        ReadStorage<'a, T>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (ent, newly_created, transition, updater): Self::SystemData) {
        for (ent, _nc, _at) in (&ent, &newly_created, &transition).join() {
            updater.insert(ent, ZeemanShiftSampler::<T>::default());
        }
    }
}

/// Calculates the Zeeman shift for each atom in each cooling beam.
#[derive(Default)]
pub struct CalculateZeemanShiftSystem<T>(PhantomData<T>) where T : TransitionComponent;
impl<'a, T> System<'a> for CalculateZeemanShiftSystem<T> where T : TransitionComponent {
    type SystemData = (
        WriteStorage<'a, ZeemanShiftSampler<T>>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, T>,
    );

    fn run(
        &mut self,
        (mut zeeman_sampler, magnetic_field_sampler, atomic_transition): Self::SystemData,
    ) {
        use rayon::prelude::*;

        (
            &mut zeeman_sampler,
            &magnetic_field_sampler,
            &atomic_transition,
        )
            .par_join()
            .for_each(|(zeeman, magnetic_field, _transition)| {
                zeeman.sigma_plus = T::mup() / HBAR * magnetic_field.magnitude;
                zeeman.sigma_minus = T::mum() / HBAR * magnetic_field.magnitude;
                zeeman.sigma_pi = T::muz() / HBAR * magnetic_field.magnitude;
            });
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::{constant::HBAR, species::Strontium88_461, laser_cooling::transition::AtomicTransition};
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;
    use nalgebra::{Matrix3, Vector3};

    #[test]
    fn test_calculate_zeeman_shift_system() {
        let mut test_world = World::new();
        test_world.register::<MagneticFieldSampler>();
        test_world.register::<Strontium88_461>();
        test_world.register::<ZeemanShiftSampler<Strontium88_461>>();

        let atom1 = test_world
            .create_entity()
            .with(MagneticFieldSampler {
                field: Vector3::new(0.0, 0.0, 1.0),
                magnitude: 1.0,
                gradient: Vector3::new(0.0, 0.0, 0.0),
                jacobian: Matrix3::zeros(),
            })
            .with(ZeemanShiftSampler::<Strontium88_461>::default())
            .with(Strontium88_461)
            .build();

        let mut system = CalculateZeemanShiftSystem::<Strontium88_461>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<ZeemanShiftSampler<Strontium88_461>>();

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .sigma_plus,
                Strontium88_461::mup() / HBAR * 1.0,
            1e-5_f64
        );

        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .sigma_minus,
                Strontium88_461::mum() / HBAR * 1.0,
            1e-5_f64
        );
        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .sigma_pi,
                Strontium88_461::muz() / HBAR * 1.0,
            1e-5_f64
        );
    }
}
