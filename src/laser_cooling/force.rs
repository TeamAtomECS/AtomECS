//! Calculation of the forces exerted on the atom by the CoolingLight entities

use super::CoolingLight;
use super::transition::{TransitionComponent};
use crate::constant;
use crate::laser::gaussian::GaussianBeam;
use crate::laser::index::LaserIndex;
use crate::laser_cooling::photons_scattered::ActualPhotonsScatteredVector;
use bevy::tasks::ComputeTaskPool;
use bevy::prelude::*;
use nalgebra::Vector3;
use rand_distr;
use rand_distr::{Distribution, Normal, UnitSphere};

use crate::atom::Force;
use crate::constant::HBAR;
use crate::integrator::{Timestep, BatchSize};

use crate::laser_cooling::repump::*;

const LASER_CACHE_SIZE: usize = 16;

/// This sytem calculates the forces from absorbing photons from the CoolingLight entities.
///
/// The system assumes that the `ActualPhotonsScatteredVector` for each atom
/// s already populated with the correct terms. Furthermore, it is assumed that a
/// `CoolingLightIndex` is present and assigned for all cooling lasers, with an index
/// corresponding to the entries in the `ActualPhotonsScatteredVector` vector.
pub fn calculate_absorption_forces<const N: usize, T : TransitionComponent>(
    laser_query: Query<(&CoolingLight, &LaserIndex, &GaussianBeam)>,
    mut atom_query: Query<(&ActualPhotonsScatteredVector<T,N>, &mut Force), Without<Dark>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
    timestep: Res<Timestep>
) {
    // There are typically only a small number of lasers in a simulation.
    // For a speedup, cache the required components into thread memory,
    // so they can be distributed to parallel workers during the atom loop.
    type CachedLaser = (CoolingLight, LaserIndex, GaussianBeam);
    let mut laser_cache: Vec<CachedLaser> = Vec::new();
    for (cooling, index, gaussian) in laser_query.iter() {
        laser_cache.push((*cooling, *index, *gaussian));
    }

    // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
    for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
        let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
        let slice = &laser_cache[base_index..max_index];
        let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
        laser_array[..max_index].copy_from_slice(slice);
        let number_in_iteration = slice.len();

        atom_query.par_for_each_mut(&task_pool, batch_size.0, 
            |(scattered, mut force)| {
                for (cooling, index, gaussian) in laser_array.iter().take(number_in_iteration) {
                    let new_force = scattered.contents[index.index].scattered * HBAR
                        / timestep.delta
                        * gaussian.direction.normalize()
                        * cooling.wavenumber();
                    force.force += new_force;
                }
            }
        );
    }
}

/// A resource that indicates that the simulation should apply random forces
/// to simulate the random walk fluctuations due to spontaneous
/// emission.
#[derive(Clone, Copy)]
pub enum EmissionForceOption {
    Off,
    On(EmissionForceConfiguration),
}
impl Default for EmissionForceOption {
    fn default() -> Self {
        EmissionForceOption::On(EmissionForceConfiguration {
            explicit_threshold: 5,
        })
    }
}

/// A particular configuration that tells the `ApplyEmissionForceSystem` when to
/// switch over to averaged mode
#[derive(Clone, Copy)]
pub struct EmissionForceConfiguration {
    /// If the number of photons scattered by a specific beam during one iteration step
    /// exceeds this number, the force vector will be generated
    /// using an averaged random walk formula instead of the explicit addition of
    /// random vectors
    pub explicit_threshold: u64,
}

/// Calculates the force vector due to the spontaneous emissions in this
/// simulation step.
///
/// Only runs if `ApplyEmissionForceOption` is initialized.
///
/// Uses an internal threshold of 5 to decide if the random vektor is iteratively
/// produced or derived by random-walk formula and a single random unit vector.
pub fn calculate_emission_forces<const N: usize, T : TransitionComponent>(
    mut atom_query: Query<(&mut Force, &ActualPhotonsScatteredVector<T,N>), With<T>>,
    task_pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
    rand_opt: Option<Res<EmissionForceOption>>,
    timestep: Res<Timestep>
) {
    match rand_opt {
        None => (),
        Some(opt) => {
            match *opt {
                EmissionForceOption::Off => {}
                EmissionForceOption::On(configuration) => {
                    atom_query.par_for_each_mut(
                        &task_pool,
                        batch_size.0,
                        |(mut force, kick)| {
                            let total: u64 = kick.calculate_total_scattered();
                            let mut rng = rand::thread_rng();
                            let omega = 2.0 * constant::PI * T::frequency();
                            let force_one_kick =
                                constant::HBAR * omega / constant::C / timestep.delta;
                            if total > configuration.explicit_threshold {
                                // see HSIUNG, HSIUNG,GORDUS,1960, A Closed General Solution of the Probability Distribution Function for
                                //Three-Dimensional Random Walk Processes*
                                let normal = Normal::new(
                                    0.0,
                                    (total as f64 * force_one_kick.powf(2.0) / 3.0).powf(0.5),
                                )
                                .unwrap();

                                let force_n_kicks = Vector3::new(
                                    normal.sample(&mut rng),
                                    normal.sample(&mut rng),
                                    normal.sample(&mut rng),
                                );
                                force.force += force_n_kicks;
                            } else {
                                // explicit random walk implementation
                                for _i in 0..total {
                                    let v: [f64; 3] = UnitSphere.sample(&mut rng);
                                    force.force +=
                                        force_one_kick * Vector3::new(v[0], v[1], v[2]);
                                }
                            }
                        }
                    );
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::CoolingLight;
    use super::*;
    use crate::constant::{HBAR, PI};
    use crate::laser::index::LaserIndex;
    use crate::laser_cooling::photons_scattered::ActualPhotonsScattered;
    use crate::laser_cooling::transition::AtomicTransition;
    use crate::species::Strontium88_461;
    use assert_approx_eq::assert_approx_eq;
    extern crate nalgebra;
    use crate::laser::{gaussian, DEFAULT_BEAM_LIMIT};
    use nalgebra::Vector3;

    /// Tests the correct implementation of the `CalculateAbsorptionForceSystem`
    #[test]
    fn test_calculate_absorption_forces_system() {
        let mut test_world = World::new();

        let time_delta = 1.0e-5;

        test_world.register::<LaserIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<ActualPhotonsScatteredVector<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Force>();
        test_world.register::<Dark>();
        test_world.insert(Timestep { delta: time_delta });

        let wavelength = Strontium88_461::wavelength();
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength,
            })
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
                rayleigh_range: gaussian::calculate_rayleigh_range(&wavelength, &2.0),
                ellipticity: 0.0,
            })
            .build();

        let number_scattered = 1_000_000.0;
        let mut aps = ActualPhotonsScattered::<Strontium88_461>::default();
        aps.scattered = number_scattered;

        let atom1 = test_world
            .create_entity()
            .with(ActualPhotonsScatteredVector {
                contents: [aps; DEFAULT_BEAM_LIMIT],
            })
            .with(Force::new())
            .build();

        let mut system = CalculateAbsorptionForcesSystem::<Strontium88_461, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();

        let actual_force_x = number_scattered * HBAR * 2. * PI / wavelength / time_delta;
        assert_approx_eq!(
            sampler_storage.get(atom1).expect("entity not found").force[0],
            actual_force_x,
            1e-20_f64
        );
    }

    /// Tests the correct implementation of the `ApplyEmissionForceSystem`
    #[test]
    fn test_apply_emission_forces_system() {
        let mut test_world = World::new();

        let time_delta = 1.0e-5;

        test_world.register::<ActualPhotonsScatteredVector<Strontium88_461, { DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Force>();
        test_world.register::<Strontium88_461>();
        test_world.insert(EmissionForceOption::default());
        test_world.insert(Timestep { delta: time_delta });
        let number_scattered = 1_000_000.0;

        let mut aps = ActualPhotonsScattered::<Strontium88_461>::default();
        aps.scattered = number_scattered;
        let atom1 = test_world
            .create_entity()
            .with(ActualPhotonsScatteredVector {
                contents: [aps; DEFAULT_BEAM_LIMIT],
            })
            .with(Force::new())
            .with(Strontium88_461)
            .build();

        let mut system = ApplyEmissionForceSystem::<Strontium88_461, { DEFAULT_BEAM_LIMIT }>::default();
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();

        let max_force_total = number_scattered * 2. * PI * Strontium88_461::frequency()
            / constant::C
            * HBAR
            / time_delta;
        assert_approx_eq!(
            sampler_storage
                .get(atom1)
                .expect("entity not found")
                .force
                .norm(),
            max_force_total / 2.0,
            // the outcome is random and will be somewhere between 0 and max_force_total
            max_force_total / 1.9
        );
    }
}
