extern crate specs;
use crate::atom::{Atom, AtomInfo};
use crate::constant;
use rand::Rng;
use specs::{Join, ReadExpect, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use super::sampler::LaserSamplers;
use crate::maths;
use nalgebra::Vector3;

use crate::atom::Force;
use crate::constant::{HBAR, PI};
use crate::integrator::Timestep;
use crate::magnetic::MagneticFieldSampler;

/// This sytem calculates the forces exerted by `CoolingLight` on entities.
///
/// The system assumes that the `LaserSamplers` and `MagneticFieldSampler` for each atom
/// are already populated with the correct terms. Furthermore, it is assumed that a
/// `CoolingLightIndex` is present and assigned for all cooling lasers, with an index
/// corresponding to the entries in the `LaserSamplers` vector.
pub struct CalculateCoolingForcesSystem;
impl<'a> System<'a> for CalculateCoolingForcesSystem {
    type SystemData = (
        ReadStorage<'a, MagneticFieldSampler>,
        WriteStorage<'a, LaserSamplers>,
        ReadStorage<'a, AtomInfo>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (magnetic_samplers, mut laser_samplers, atom_info, mut forces): Self::SystemData,
    ) {
        // Outer loop over atoms
        for (atom_info, bfield, laser_samplers, mut force) in (
            &atom_info,
            &magnetic_samplers,
            &mut laser_samplers,
            &mut forces,
        )
            .join()
        {
            // Inner loop over cooling lasers
            for mut laser_sampler in &mut laser_samplers.contents {
                //let s0 = 50.0;
                let s0 = laser_sampler.intensity / atom_info.saturation_intensity;
                let angular_detuning = (laser_sampler.wavevector.norm() * constant::C / 2. / PI
                    - atom_info.frequency
                    - laser_sampler.doppler_shift)
                    * 2.0
                    * PI;
                let wavevector = laser_sampler.wavevector.clone();
                let costheta = wavevector.normalize().dot(&bfield.field.normalize());
                let gamma = atom_info.gamma();
                let scatter1 = 0.25 * (laser_sampler.polarization * costheta + 1.).powf(2.) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (angular_detuning - atom_info.mup / HBAR * bfield.magnitude)
                            .powf(2.)
                            / gamma.powf(2.));
                let scatter2 = 0.25 * (laser_sampler.polarization * costheta - 1.).powf(2.) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (angular_detuning - atom_info.mum / HBAR * bfield.magnitude)
                            .powf(2.)
                            / gamma.powf(2.));
                let scatter3 = 0.5 * (1. - costheta.powf(2.)) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (angular_detuning - atom_info.muz / HBAR * bfield.magnitude)
                            .powf(2.)
                            / gamma.powf(2.));
                let cooling_force = wavevector * s0 * HBAR * (scatter1 + scatter2 + scatter3);
                laser_sampler.force = cooling_force.clone();
                println!("detuning{}", angular_detuning / gamma);
                force.force = force.force + cooling_force;
            }
        }
    }
}

pub struct RandomWalkSystem;

impl<'a> System<'a> for RandomWalkSystem {
    type SystemData = (
        WriteStorage<'a, Force>,
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomInfo>,
        ReadExpect<'a, Timestep>,
    );

    fn run(&mut self, (mut force, samplers, _atom, atom_info, timestep): Self::SystemData) {
        for (mut force, samplers, _, atom_info) in
            (&mut force, &samplers, &_atom, &atom_info).join()
        {
            let mut total_force = 0.;
            let omega = 2.0 * constant::PI * atom_info.frequency;
            for sampler in samplers.contents.iter() {
                total_force = total_force + sampler.force.norm();
            }
            let force_one_atom = constant::HBAR * omega / timestep.delta;
            let mut number_collision = total_force / force_one_atom;
            //println!("{}", number_collision);
            let mut force_real = Vector3::new(0., 0., 0.);
            let mut rng = rand::thread_rng();
            loop {
                if number_collision > 1. {
                    force_real = force_real + force_one_atom * maths::random_direction();
                    number_collision = number_collision - 1.;
                } else {
                    let luck = rng.gen_range(0.0, 1.0);
                    if luck < number_collision {
                        force_real = force_real + force_one_atom * maths::random_direction();
                        break;
                    } else {
                        break;
                    }
                }
            }
            force.force = force.force + force_real;
        }
    }
}
#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::constant;
    use crate::laser::sampler::{LaserSampler, LaserSamplers};
    use crate::magnetic::MagneticFieldSampler;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    /// Test cooling force calculations
    #[test]
    fn test_cooling_force() {
        let rb = AtomInfo::rubidium();

        let lambda = constant::C / rb.frequency;
        let wavevector = Vector3::new(1.0, 0.0, 0.0) * 2.0 * constant::PI / lambda;
        let b_field = MagneticFieldSampler::tesla(Vector3::new(1.0e-6, 0.0, 0.0));
        {
            // Test that the force goes to zero when intensity is zero.
            let doppler_shift = 0.0;
            let intensity = 0.0;
            let force = calculate_cooling_force(wavevector, intensity, doppler_shift, 1.0, b_field);
            assert_eq!(force[0], 0.0);
            assert_eq!(force[1], 0.0);
            assert_eq!(force[2], 0.0);
        }

        {
            // Test that the force goes to zero in the limit of large detuning
            let doppler_shift = 1.0e16;
            let intensity = rb.saturation_intensity;
            let force = calculate_cooling_force(wavevector, intensity, doppler_shift, 1.0, b_field);
            assert_approx_eq!(force[0], 0.0, 1.0e-38);
            assert_eq!(force[1], 0.0);
            assert_eq!(force[2], 0.0);
        }

        {
            // Test that force pushes away from laser beam
            let doppler_shift = 0.0;
            let intensity = rb.saturation_intensity;
            let force = calculate_cooling_force(wavevector, intensity, doppler_shift, 1.0, b_field);
            assert_eq!(force[0] > 0.0, true);
            assert_eq!(force[1], 0.0);
            assert_eq!(force[2], 0.0);
        }

        {
            // Test force calculation on resonance
            let doppler_shift = 0.0;
            let intensity = rb.saturation_intensity;

            let photon_momentum = constant::HBAR * wavevector;
            let i_norm = 1.0;
            let scattering_rate = (AtomInfo::rubidium().gamma() / 2.0) * i_norm / (1.0 + i_norm);
            let f_scatt = photon_momentum * scattering_rate;

            let force = calculate_cooling_force(wavevector, intensity, doppler_shift, 1.0, b_field);
            assert_approx_eq!(force[0] / f_scatt[0], 1.0, 0.01);
            assert_eq!(force[1], 0.0);
            assert_eq!(force[2], 0.0);
        }

        {
            // Test force calculation detuned by one gamma at Isat

        }

        {
            // Test that scattering rate goes to Gamma/2 in the limit of saturation

            // let doppler_shift = 0.0;
            // let intensity = 1000.0 * rb.saturation_intensity;
            // let force = calculate_cooling_force(wavevector, intensity, doppler_shift, 1.0, b_field);
            // assert_eq!(force[0] > 0.0, true);
            // assert_eq!(force[1], 0.0);
            // assert_eq!(force[2], 0.0);
        }

        {
            // Test that scattering rate goes to zero when I=0.
        }

        {
            // Test that scattering rate goes to zero at large detuning.
        }

        {
            // Test correct value of scattering rate when I=Isat, delta=Gamma.
        }
    }

    /// Uses the `CalculateCoolingForcesSystem` to calculate the force exerted on an atom.
    fn calculate_cooling_force(
        wavevector: Vector3<f64>,
        intensity: f64,
        doppler_shift: f64,
        polarization: f64,
        b_field: MagneticFieldSampler,
    ) -> Vector3<f64> {
        let mut test_world = World::new();
        test_world.register::<AtomInfo>();
        test_world.register::<MagneticFieldSampler>();
        test_world.register::<Force>();
        test_world.register::<LaserSamplers>();

        let atom1 = test_world
            .create_entity()
            .with(Force::new())
            .with(LaserSamplers {
                contents: vec![LaserSampler {
                    force: Vector3::new(0.0, 0.0, 0.0),
                    polarization: polarization,
                    wavevector: wavevector,
                    intensity: intensity,
                    doppler_shift: doppler_shift,
                    scattering_rate: 0.0,
                }],
            })
            .with(b_field)
            .with(AtomInfo::rubidium())
            .build();

        let mut system = CalculateCoolingForcesSystem {};
        system.run_now(&test_world.res);

        // See eg Foot, Atomic Physics, p180.
        let force_storage = test_world.read_storage::<Force>();
        force_storage.get(atom1).expect("entity not found").force
    }
}
