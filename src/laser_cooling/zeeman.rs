//! Shift in an atom's transition frequency due to a magnetic field (zeeman effect)
use std::marker::PhantomData;

use crate::integrator::BatchSize;
use crate::magnetic::MagneticFieldSampler;
use crate::constant::HBAR;
use crate::initiate::NewlyCreated;
use bevy::tasks::ComputeTaskPool;
use serde::Serialize;
use bevy::prelude::*;

use super::transition::TransitionComponent;

/// Represents the (angular) Zeeman shift of the atom depending on the magnetic field it experiences
#[derive(Clone, Copy, Serialize, Component)]
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

/// Attaches the [ZeemanShifSampler] component to newly created atoms.
pub fn attach_zeeman_shift_samplers_to_newly_created_atoms<T>(
    query: Query<Entity, (With<NewlyCreated>, With<T>)>,
    mut commands: Commands
) where T : TransitionComponent {
    for entity in query.iter() {
        commands.entity(entity).insert(ZeemanShiftSampler::<T>::default());
    }
}

/// Calculates the Zeeman shift for each atom in each cooling beam.
pub fn calculate_zeeman_shift<T>(
    mut query: Query<(&mut ZeemanShiftSampler<T>, &MagneticFieldSampler), With<T>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>
) where T : TransitionComponent {

    query.par_for_each_mut(
        &task_pool,
        batch_size.0,
        |(
            mut zeeman,
            magnetic_field,
            )| {
            zeeman.sigma_plus = T::mup() / HBAR * magnetic_field.magnitude;
            zeeman.sigma_minus = T::mum() / HBAR * magnetic_field.magnitude;
            zeeman.sigma_pi = T::muz() / HBAR * magnetic_field.magnitude;
        }
    );
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::{constant::HBAR, species::Strontium88_461, laser_cooling::transition::AtomicTransition};
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::{Matrix3, Vector3};

    #[test]
    fn test_calculate_zeeman_shift_system() {
        let mut app = App::new();

        let atom = app.world
            .spawn()
            .insert(MagneticFieldSampler {
                field: Vector3::new(0.0, 0.0, 1.0),
                magnitude: 1.0,
                gradient: Vector3::new(0.0, 0.0, 0.0),
                jacobian: Matrix3::zeros(),
            })
            .insert(ZeemanShiftSampler::<Strontium88_461>::default())
            .insert(Strontium88_461)
            .id();

        app.add_system(calculate_zeeman_shift);
        app.update();

        let result = app.world
            .entity(atom)
            .get::<ZeemanShiftSampler<Strontium88_461>>()
            .expect("entity not found");
            
        assert_approx_eq!(
            result.sigma_plus,
                Strontium88_461::mup() / HBAR * 1.0,
            1e-5_f64
        );

        assert_approx_eq!(
            result.sigma_minus,
                Strontium88_461::mum() / HBAR * 1.0,
            1e-5_f64
        );
        assert_approx_eq!(
            result.sigma_pi,
                Strontium88_461::muz() / HBAR * 1.0,
            1e-5_f64
        );
    }

    fn test_attach_zeeman_sampler_to_newly_created_atoms() {
        let mut app = App::new();

        let atom = app.world
            .spawn()
            .insert(NewlyCreated)
            .insert(Strontium88_461)
            .id();

        app.add_system(attach_zeeman_shift_samplers_to_newly_created_atoms::<Strontium88_461>);
        app.update();

        assert!(
            let result = app.world
            .entity(atom)
            .contains::<ZeemanShiftSampler<Strontium88_461>>()
        );
    }
}
