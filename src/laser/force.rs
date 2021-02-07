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
        for (scattered, mut force, _dark) in (&actual_scattered_vector, &mut forces, !&_dark).join()
        {
            for (index, cooling, gaussian) in
                (&cooling_index, &cooling_light, &gaussian_beam).join()
            {
                let new_force = scattered.contents[index.index].scattered as f64 * HBAR
                    / timestep.delta
                    * gaussian.direction.normalize()
                    * cooling.wavenumber();
                force.force = force.force + new_force;
            }
        }
    }
}

/// A resource that indicates that the simulation should apply random forces
/// to simulate the random walk fluctuations due to spontaneous
/// emission.
pub struct ApplyEmissionForceOption;

pub struct ApplyEmissionForceSystem;

impl<'a> System<'a> for ApplyEmissionForceSystem {
    type SystemData = (
        Option<Read<'a, ApplyEmissionForceOption>>,
        WriteStorage<'a, Force>,
        ReadStorage<'a, ActualPhotonsScatteredVector>,
        ReadStorage<'a, AtomicTransition>,
        ReadExpect<'a, Timestep>,
    );

    fn run(
        &mut self,
        (rand_opt, mut force, actual_scattered_vector, atom_info, timestep): Self::SystemData,
    ) {
        match rand_opt {
            None => (),
            Some(_rand) => {
                for (mut force, atom_info, kick) in
                    (&mut force, &atom_info, &actual_scattered_vector).join()
                {
                    let total: u64 = kick.calculate_total_scattered();
                    let mut rng = rand::thread_rng();
                    let omega = 2.0 * constant::PI * atom_info.frequency;
                    let force_one_kick = constant::HBAR * omega / constant::C / timestep.delta;
                    if total > 5 {
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
                            force.force = force.force + force_one_kick * maths::random_direction();
                        }
                    }
                }
            }
        }
    }
}
