//! A module that implements systems and components for calculating optical scattering forces in AtomECS.

use std::marker::PhantomData;
use crate::constant;
use crate::initiate::NewlyCreated;
use crate::ramp::Lerp;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use transition::AtomicTransition;

use self::{transition::TransitionComponent, photons_scattered::ScatteringFluctuationsOption};

pub mod doppler;
pub mod force;
pub mod photons_scattered;
pub mod rate;
pub mod repump;
pub mod sampler;
pub mod twolevel;
pub mod transition;
pub mod zeeman;
mod sampler_masks;

/// A component representing light properties used for laser cooling.
///
/// Holds information about polarization and wavelength
/// and works as a marker for all laser cooling processes.
#[derive(Deserialize, Serialize, Clone, Copy, Component)]
#[component(storage = "SparseSet")]
pub struct CoolingLight {
    /// Polarisation of the laser light, 1 for +, -1 for -,
    ///
    /// Note that the polarization is defined by the quantization vector (e.g. magnetic field)
    /// and not (always) in direction of the wavevector. Look at the given examples of 3D-MOT
    /// simulations to see a working example if unsure.
    ///
    /// Currently this is an integer value since every partial polarization can be expressed
    /// as a superposition of fully polarized beams. It  is possible that this will be
    /// changed to a non-integer value in the future.
    pub polarization: i32,

    /// wavelength of the laser light, in SI units of m.
    pub wavelength: f64,
}
impl Lerp<CoolingLight> for CoolingLight {
    fn lerp(&self, b: &CoolingLight, amount: f64) -> Self {
        CoolingLight {
            polarization: self.polarization,
            wavelength: self.wavelength - (self.wavelength - b.wavelength) * amount,
        }
    }
}
impl CoolingLight {
    /// Frequency of the cooling light in units of Hz
    pub fn frequency(&self) -> f64 {
        constant::C / self.wavelength
    }

    /// Wavenumber of the cooling light, in units of 2pi inverse metres.
    pub fn wavenumber(&self) -> f64 {
        2.0 * constant::PI / self.wavelength
    }

    /// Creates a `CoolingLight` component from the desired atomic species.
    ///
    /// # Arguments
    ///
    /// * `<T>`: The atomic transition to take the base wavelength from.
    ///
    /// * `detuning`: Detuning of the laser from transition in units of MHz
    ///
    /// * `polarization`: Polarization of the cooling beam.
    pub fn for_transition<T>(detuning: f64, polarization: i32) -> Self where T : AtomicTransition {
        let freq = T::frequency() + detuning * 1.0e6;
        CoolingLight {
            wavelength: constant::C / freq,
            polarization,
        }
    }
}

/// Attaches components required for laser calculations to laser beams with a [CoolingLight] component.
pub fn attach_components_to_cooling_lasers(
    requires_query: Query<Entity, (With<CoolingLight>, Without<crate::laser::RequiresIntensityCalculation>)>,
    index_query: Query<Entity, (With<CoolingLight>, Without<crate::laser::index::LaserIndex>)>,
    mut commands: Commands
) {
    for e in requires_query.iter() {
        commands.entity(e).insert(crate::laser::RequiresIntensityCalculation);
    }
    for e in index_query.iter() {
        commands.entity(e).insert(crate::laser::index::LaserIndex::default());
    }
}

/// A system which attaches components required for optical scattering force calculation to newly created atoms.
///
/// They are recognized as newly created if they are associated with
/// the `NewlyCreated` component.
pub fn attach_components_to_newly_created_atoms<const N: usize, T>(
    query: Query<Entity, (With<NewlyCreated>, With<T>)>,
    mut commands: Commands
) 
where T : TransitionComponent
{
    for entity in query.iter() {
        commands.entity(entity)
            .insert(doppler::DopplerShiftSamplers {
                contents: [doppler::DopplerShiftSampler::default(); N],
            })
            .insert(sampler::LaserDetuningSamplers::<T,N> {
                contents: [sampler::LaserDetuningSampler::default(); N],
            })
            .insert(rate::RateCoefficients {
                contents: [rate::RateCoefficient::<T>::default(); N],
            })
            .insert(twolevel::TwoLevelPopulation::<T>::default())
            .insert(photons_scattered::TotalPhotonsScattered::<T>::default())
            .insert(photons_scattered::ExpectedPhotonsScatteredVector::<T,N> {
                contents: [photons_scattered::ExpectedPhotonsScattered::default(); N],
            })
            .insert(photons_scattered::ActualPhotonsScatteredVector::<T,N> {
                contents: [photons_scattered::ActualPhotonsScattered::default(); N],
            })
            .insert(sampler_masks::CoolingLaserSamplerMasks {
                contents: [sampler_masks::CoolingLaserSamplerMask::default(); N],
            });
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum LaserCoolingSystems {
    Set,
    AttachComponentsToNewlyCreatedAtoms,
    AttachComponentsToCoolingLasers,
    PopulateCoolingLightMasks,
    CalculateDopplerShift,
    CalculateZeemanShift,
    CalculateLaserDetuning,
    CalculateRateCoefficients,
    CalculateTwoLevelPopulation,
    CalculateMeanTotalPhotonsScattered,
    CalculateExpectedPhotonsScattered,
    CalculateActualPhotonsScattered,
    CalculateAbsorptionForces,
    CalculateEmissionForces,
    AttachZeemanSamplersToNewlyCreatedAtoms,
    MakeAtomsDark
}

/// This plugin performs simulations of laser cooling using a two-level rate equation approach.
/// 
/// For more information see [crate::laser_cooling].
/// 
/// # Generic Arguments
/// 
/// * `T`: The laser cooling transition to solve the two-level system for.
/// 
/// * `N`: The maximum number of laser beams (must match the `LaserPlugin`).
#[derive(Default)]
pub struct LaserCoolingPlugin<T, const N : usize>(PhantomData<T>) where T : TransitionComponent;
impl<T, const N : usize> Plugin for LaserCoolingPlugin<T, N> where T : TransitionComponent {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            // Note - not sure yet how this plays with multiple cooling transitions.#
            //  E.g., Populate cooling masks will run twice if two plugins are added? How to guarantee safety?
            //  Maybe it is automatically assured because they will have the same labels?
            //  How generally to ensure the parallel systems all operate in the right order?
            //  Need to make sure each <T> 'stays in its lane', e.g. only writes values for those lasers.
            SystemSet::new()    
            .label(LaserCoolingSystems::Set)
            .with_system(
                attach_components_to_newly_created_atoms::<N, T>
                .label(LaserCoolingSystems::AttachComponentsToNewlyCreatedAtoms)
            )
            .with_system(
                zeeman::attach_zeeman_shift_samplers_to_newly_created_atoms::<T>
                .label(LaserCoolingSystems::AttachZeemanSamplersToNewlyCreatedAtoms)
            )
            .with_system(
                attach_components_to_cooling_lasers
                .label(LaserCoolingSystems::AttachComponentsToCoolingLasers)
            )
            .with_system(
                sampler_masks::populate_cooling_light_masks::<N>
                .label(LaserCoolingSystems::PopulateCoolingLightMasks)
                .after(crate::laser::LaserSystems::IndexLasers)
            )
            .with_system(
                doppler::calculate_doppler_shift::<N>
                .label(LaserCoolingSystems::CalculateDopplerShift)
                .after(crate::laser::LaserSystems::IndexLasers)
            )
            .with_system(
                zeeman::calculate_zeeman_shift::<T>
                .label(LaserCoolingSystems::CalculateZeemanShift)
                .after(crate::magnetic::MagneticSystems::CalculateMagneticFieldMagnitude)
            )
            .with_system(
                sampler::calculate_laser_detuning::<N,T>
                .label(LaserCoolingSystems::CalculateLaserDetuning)
                .after(LaserCoolingSystems::CalculateZeemanShift)
                .after(LaserCoolingSystems::CalculateDopplerShift)
            )
            .with_system(
                rate::calculate_rate_coefficients::<N,T>
                .label(LaserCoolingSystems::CalculateRateCoefficients)
                .after(LaserCoolingSystems::CalculateLaserDetuning)
            )
            .with_system(
                twolevel::calculate_two_level_population::<N, T>
                .label(LaserCoolingSystems::CalculateTwoLevelPopulation)
                .after(LaserCoolingSystems::CalculateRateCoefficients)
                .after(LaserCoolingSystems::PopulateCoolingLightMasks)
            )
            .with_system(
                photons_scattered::calculate_mean_total_photons_scattered::<T>
                .label(LaserCoolingSystems::CalculateMeanTotalPhotonsScattered)
                .after(LaserCoolingSystems::CalculateTwoLevelPopulation)
            )
            .with_system(
                photons_scattered::calculate_expected_photons_scattered::<N, T>
                .label(LaserCoolingSystems::CalculateExpectedPhotonsScattered)
                .after(LaserCoolingSystems::CalculateMeanTotalPhotonsScattered)
            )
            .with_system(
                photons_scattered::calculate_actual_photons_scattered::<N, T>
                .label(LaserCoolingSystems::CalculateActualPhotonsScattered)
                .after(LaserCoolingSystems::CalculateExpectedPhotonsScattered)
            )
            .with_system(
                force::calculate_absorption_forces::<N, T>
                .label(LaserCoolingSystems::CalculateAbsorptionForces)
                .after(LaserCoolingSystems::CalculateActualPhotonsScattered)
            )
            .with_system(
                force::calculate_emission_forces::<N, T>
                .label(LaserCoolingSystems::CalculateAbsorptionForces)
                .after(LaserCoolingSystems::CalculateAbsorptionForces)
            )
            .with_system(
                repump::make_atoms_dark::<T>
                .label(LaserCoolingSystems::MakeAtomsDark)
                .after(LaserCoolingSystems::CalculateAbsorptionForces)
            )
        );
        app.world.init_resource::<ScatteringFluctuationsOption>()
    }
}

#[cfg(test)]
pub mod tests {

    use crate::species::Rubidium87_780D2;

    use super::*;
    use assert_approx_eq::assert_approx_eq;
    #[test]
    fn test_add_index_component_to_cooling_lights() {
        let mut test_world = World::new();
        test_world.register::<LaserIndex>();
        test_world.register::<CoolingLight>();

        let test_entity = test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength: 780e-9,
            })
            .build();

        let mut system = AttachIndexToCoolingLightSystem;
        system.run_now(&test_world);
        test_world.maintain();

        assert!(test_world
            .read_storage::<LaserIndex>()
            .get(test_entity)
            .is_some());
    }

    #[test]
    fn test_for_species() {
        let detuning = 12.0;
        let light = CoolingLight::for_transition::<Rubidium87_780D2>(detuning, 1);
        assert_approx_eq!(
            light.frequency(),
            Rubidium87_780D2::frequency() + 1.0e6 * detuning
        );
    }
}
