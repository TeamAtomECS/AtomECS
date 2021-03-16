//! Calculation of the forces exerted on the atom by the CoolingLight entities

extern crate rayon;
extern crate specs;
use crate::atom::AtomicTransition;
use crate::constant;
use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
use crate::laser::gaussian::GaussianBeam;
use crate::laser::photons_scattered::ActualPhotonsScatteredVector;
use crate::maths;
use rand::distributions::{Distribution, Normal};
use specs::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use nalgebra::Vector3;

use crate::atom::Force;
use crate::constant::HBAR;
use crate::integrator::Timestep;

use crate::laser::repump::*;

const LASER_CACHE_SIZE: usize = 16;

/// This sytem calculates the forces from absorbing photons from the CoolingLight entities.
///
/// The system assumes that the `ActualPhotonsScatteredVector` for each atom
/// s already populated with the correct terms. Furthermore, it is assumed that a
/// `CoolingLightIndex` is present and assigned for all cooling lasers, with an index
/// corresponding to the entries in the `ActualPhotonsScatteredVector` vector.
pub struct CalculateAbsorptionForcesSystem;
impl<'a> System<'a> for CalculateAbsorptionForcesSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, ActualPhotonsScatteredVector>,
        WriteStorage<'a, Force>,
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, Dark>,
    );

    fn run(
        &mut self,
        (
            cooling_index,
            cooling_light,
            gaussian_beam,
            actual_scattered_vector,
            mut forces,
            timestep,
            _dark,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        // There are typically only a small number of lasers in a simulation.
        // For a speedup, cache the required components into thread memory,
        // so they can be distributed to parallel workers during the atom loop.
        type CachedLaser = (CoolingLight, CoolingLightIndex, GaussianBeam);
        let laser_cache: Vec<CachedLaser> = (&cooling_light, &cooling_index, &gaussian_beam)
            .join()
            .map(|(cooling, index, gaussian)| (cooling.clone(), index.clone(), gaussian.clone()))
            .collect();

        // Perform the iteration over atoms, `LASER_CACHE_SIZE` at a time.
        for base_index in (0..laser_cache.len()).step_by(LASER_CACHE_SIZE) {
            let max_index = laser_cache.len().min(base_index + LASER_CACHE_SIZE);
            let slice = &laser_cache[base_index..max_index];
            let mut laser_array = vec![laser_cache[0]; LASER_CACHE_SIZE];
            laser_array[..max_index].copy_from_slice(slice);
            let number_in_iteration = slice.len();

            (&actual_scattered_vector, &mut forces, !&_dark)
                .par_join()
                .for_each(|(scattered, mut force, _)| {
                    for i in 0..number_in_iteration {
                        let (cooling, index, gaussian) = laser_array[i];
                        let new_force = scattered.contents[index.index].scattered as f64 * HBAR
                            / timestep.delta
                            * gaussian.direction.normalize()
                            * cooling.wavenumber();
                        force.force = force.force + new_force;
                    }
                })
        }
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
pub struct ApplyEmissionForceSystem;

impl<'a> System<'a> for ApplyEmissionForceSystem {
    type SystemData = (
        Option<Read<'a, EmissionForceOption>>,
        WriteStorage<'a, Force>,
        ReadStorage<'a, ActualPhotonsScatteredVector>,
        ReadStorage<'a, AtomicTransition>,
        ReadExpect<'a, Timestep>,
    );

    fn run(
        &mut self,
        (rand_opt, mut force, actual_scattered_vector, atom_info, timestep): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        match rand_opt {
            None => (),
            Some(opt) => {
                match *opt {
                    EmissionForceOption::Off => {}
                    EmissionForceOption::On(configuration) => {
                        (&mut force, &atom_info, &actual_scattered_vector)
                            .par_join()
                            .for_each(|(mut force, atom_info, kick)| {
                                let total: u64 = kick.calculate_total_scattered();
                                let mut rng = rand::thread_rng();
                                let omega = 2.0 * constant::PI * atom_info.frequency;
                                let force_one_kick =
                                    constant::HBAR * omega / constant::C / timestep.delta;
                                if total > configuration.explicit_threshold {
                                    // see HSIUNG, HSIUNG,GORDUS,1960, A Closed General Solution of the Probability Distribution Function for
                                    //Three-Dimensional Random Walk Processes*
                                    let normal = Normal::new(
                                        0.0,
                                        (total as f64 * force_one_kick.powf(2.0) / 3.0).powf(0.5),
                                    );

                                    let force_n_kicks = Vector3::new(
                                        normal.sample(&mut rng),
                                        normal.sample(&mut rng),
                                        normal.sample(&mut rng),
                                    );
                                    force.force = force.force + force_n_kicks;
                                } else {
                                    // explicit random walk implementation
                                    for _i in 0..total {
                                        force.force = force.force
                                            + force_one_kick * maths::random_direction();
                                    }
                                }
                            });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::constant::{HBAR, PI};
    use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Tests the correct implementation of the `CalculateAbsorptionForceSystem`
    #[test]
    fn test_calculate_absorption_forces_system() {
        let mut test_world = World::new();

        let time_delta = 1.0e-5;

        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<ActualPhotonsScatteredVector>();
        test_world.register::<Force>();
        test_world.register::<Dark>();
        test_world.add_resource(Timestep { delta: time_delta });

        let wavelength = 461e-9;
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1,
                wavelength: wavelength,
            })
            .with(CoolingLightIndex {
                index: 0,
                initiated: true,
            })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
            })
            .build();

        let number_scattered = 1_000_000.0;

        let atom1 = test_world
            .create_entity()
            .with(ActualPhotonsScatteredVector {
                contents: [crate::laser::photons_scattered::ActualPhotonsScattered {
                    scattered: number_scattered,
                }; crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(Force::new())
            .build();

        let mut system = CalculateAbsorptionForcesSystem;
        system.run_now(&test_world.res);
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

        test_world.register::<ActualPhotonsScatteredVector>();
        test_world.register::<Force>();
        test_world.register::<AtomicTransition>();
        test_world.add_resource(EmissionForceOption::default());
        test_world.add_resource(Timestep { delta: time_delta });
        let number_scattered = 1_000_000.0;

        let atom1 = test_world
            .create_entity()
            .with(ActualPhotonsScatteredVector {
                contents: [crate::laser::photons_scattered::ActualPhotonsScattered {
                    scattered: number_scattered,
                }; crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(Force::new())
            .with(AtomicTransition::strontium())
            .build();

        let mut system = ApplyEmissionForceSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();

        let max_force_total = number_scattered * 2. * PI * AtomicTransition::strontium().frequency
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
