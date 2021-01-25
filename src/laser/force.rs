extern crate rayon;
extern crate specs;
use crate::atom::AtomicTransition;
use crate::constant;
use crate::laser::photons_scattered::ActualPhotonsScatteredVector;
use crate::laser::sampler::LaserDetuningSamplers;
use crate::maths;
use rand::distributions::{Distribution, Normal};
use specs::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use super::intensity::LaserIntensitySamplers;
use super::sampler::LaserSamplers;
use nalgebra::Vector3;

use crate::atom::Force;
use crate::constant::HBAR;
use crate::integrator::Timestep;
use crate::magnetic::MagneticFieldSampler;

use crate::laser::repump::*;

/// This sytem calculates the forces exerted by `CoolingLight` on entities.
///
/// The system assumes that the `LaserSamplers` and `MagneticFieldSampler` for each atom
/// are already populated with the correct terms. Furthermore, it is assumed that a
/// `CoolingLightIndex` is present and assigned for all cooling lasers, with an index
/// corresponding to the entries in the `LaserSamplers` vector.
pub struct CalculateCoolingForcesSystem;
impl<'a> System<'a> for CalculateCoolingForcesSystem {
    type SystemData = (
        ReadStorage<'a, LaserDetuningSamplers>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, LaserIntensitySamplers>,
        WriteStorage<'a, LaserSamplers>,
        ReadStorage<'a, AtomicTransition>,
        WriteStorage<'a, Force>,
        ReadStorage<'a, Dark>,
    );

    fn run(
        &mut self,
        (
            laser_detuning_samplers,
            magnetic_samplers,
            intensity_samplers,
            mut laser_samplers,
            atom_info,
            mut forces,
            _dark,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (
            &laser_detuning_samplers,
            &atom_info,
            &magnetic_samplers,
            &intensity_samplers,
            &mut laser_samplers,
            &mut forces,
            !&_dark,
        )
            .par_join()
            .for_each(
                |(
                    laser_detuning_samplers,
                    atom_info,
                    bfield,
                    intensity_samplers,
                    laser_samplers,
                    mut force,
                    (),
                )| {
                    // Inner loop over cooling lasers
                    for count in 0..laser_samplers.contents.len() {
                        //let s0 = 1.0;
                        let s0 = intensity_samplers.contents[count].intensity
                            / atom_info.saturation_intensity;
                        //println!("s0 : {}", s0);
                        //println!("laserfre{},atomfre{},shift {}",laser_sampler.wavevector.norm() * constant::C / 2. / PI,atom_info.frequency,laser_sampler.doppler_shift);
                        let wavevector = laser_samplers.contents[count].wavevector.clone();
                        let costheta = if &bfield.field.norm_squared() < &(10.0 * f64::EPSILON) {
                            0.0
                        } else {
                            wavevector.normalize().dot(&bfield.field.normalize())
                        };
                        let gamma = atom_info.gamma();
                        let scatter1 = 0.25
                            * (laser_samplers.contents[count].polarization * costheta + 1.)
                                .powf(2.)
                            * gamma
                            / 2.
                            / (1.
                                + s0
                                + 4. * (laser_detuning_samplers.contents[count]
                                    .detuning_sigma_plus)
                                    .powf(2.)
                                    / gamma.powf(2.));
                        let scatter2 = 0.25
                            * (laser_samplers.contents[count].polarization * costheta - 1.)
                                .powf(2.)
                            * gamma
                            / 2.
                            / (1.
                                + s0
                                + 4. * (laser_detuning_samplers.contents[count]
                                    .detuning_sigma_minus)
                                    .powf(2.)
                                    / gamma.powf(2.));
                        let scatter3 = 0.5 * (1. - costheta.powf(2.)) * gamma
                            / 2.
                            / (1.
                                + s0
                                + 4. * (laser_detuning_samplers.contents[count].detuning_sigma_pi)
                                    .powf(2.)
                                    / gamma.powf(2.));
                        let cooling_force =
                            wavevector * s0 * HBAR * (scatter1 + scatter2 + scatter3);
                        laser_samplers.contents[count].force = cooling_force.clone();
                        //println!("detuning{}", angular_detuning / gamma);
                        force.force = force.force + cooling_force;
                    }
                },
            );
    }
}

/// A resource that indicates that the simulation should apply random forces to simulate fluctuations in the number of scattered photons.
pub struct RandomScatteringForceOption;

pub struct ApplyEmissionForceSystem;

impl<'a> System<'a> for ApplyEmissionForceSystem {
    type SystemData = (
        Option<Read<'a, RandomScatteringForceOption>>,
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
