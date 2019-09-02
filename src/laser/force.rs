extern crate specs;
use crate::atom::{Atom, AtomInfo};
use crate::constant;
use rand::Rng;
use specs::{Component, Join, ReadExpect, ReadStorage, System, VecStorage, WriteStorage};
extern crate nalgebra;
use super::sampler::LaserSamplers;
use crate::maths;
use nalgebra::Vector3;

use crate::atom::Force;
use crate::constant::{HBAR, PI};
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
        ReadStorage<'a, MagneticFieldSampler>,
        WriteStorage<'a, LaserSamplers>,
        ReadStorage<'a, AtomInfo>,
        WriteStorage<'a, Force>,
        ReadStorage<'a, Dark>,
    );

    fn run(
        &mut self,
        (magnetic_samplers, mut laser_samplers, atom_info, mut forces, _dark): Self::SystemData,
    ) {
        // Outer loop over atoms
        for (atom_info, bfield, laser_samplers, mut force, ()) in (
            &atom_info,
            &magnetic_samplers,
            &mut laser_samplers,
            &mut forces,
            !&_dark,
        )
            .join()
        {
            // Inner loop over cooling lasers
            for mut laser_sampler in &mut laser_samplers.contents {
                //let s0 = 1.0;
                let s0 = laser_sampler.intensity / atom_info.saturation_intensity;
                //println!("s0 : {}", s0);
                let angular_detuning = (laser_sampler.wavevector.norm() * constant::C / 2. / PI
                    - atom_info.frequency)
                    * 2.0
                    * PI
                    - laser_sampler.doppler_shift;
                //println!("laserfre{},atomfre{},shift {}",laser_sampler.wavevector.norm() * constant::C / 2. / PI,atom_info.frequency,laser_sampler.doppler_shift);
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
                //println!("detuning{}", angular_detuning / gamma);
                force.force = force.force + cooling_force;
            }
        }
    }
}

pub struct NumberKick {
    pub value: i32,
}

impl Component for NumberKick {
    type Storage = VecStorage<Self>;
}

pub struct RandomWalkMarker {
    pub value: bool,
}

pub struct CalculateKickSystem;

impl<'a> System<'a> for CalculateKickSystem {
    type SystemData = (
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomInfo>,
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, Dark>,
        WriteStorage<'a, NumberKick>,
    );

    fn run(&mut self, (samplers, _atom, atom_info, timestep, _dark, mut number): Self::SystemData) {
        for (samplers, _, atom_info, (), num) in
            (&samplers, &_atom, &atom_info, !&_dark, &mut number).join()
        {
            num.value = 0;
            let mut total_force = 0.;
            let omega = 2.0 * constant::PI * atom_info.frequency;
            for sampler in samplers.contents.iter() {
                total_force = total_force + sampler.force.norm();
            }
            let force_one_atom = constant::HBAR * omega / constant::C / timestep.delta;
            let mut number_collision = total_force / force_one_atom;
            //println!("collision{}", number_collision);
            let mut rng = rand::thread_rng();
            loop {
                if number_collision > 1. {
                    num.value = num.value + 1;
                    number_collision = number_collision - 1.;
                } else {
                    let luck = rng.gen_range(0.0, 1.0);
                    if luck < number_collision {
                        num.value = num.value + 1;
                        break;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
pub struct RandomWalkSystem;

impl<'a> System<'a> for RandomWalkSystem {
    type SystemData = (
        ReadExpect<'a, RandomWalkMarker>,
        WriteStorage<'a, Force>,
        ReadStorage<'a, NumberKick>,
        ReadStorage<'a, AtomInfo>,
        ReadExpect<'a, Timestep>,
    );

    fn run(&mut self, (rand, mut force, kick, atom_info, timestep): Self::SystemData) {
        if rand.value {
            for (mut force, atom_info, kick) in (&mut force, &atom_info, &kick).join() {
                let omega = 2.0 * constant::PI * atom_info.frequency;
                let force_one_atom = constant::HBAR * omega / constant::C / timestep.delta;
                let mut force_real = Vector3::new(0., 0., 0.);
                for _i in 0..kick.value {
                    force_real = force_real + force_one_atom * maths::random_direction();
                }
                force.force = force.force + force_real;
            }
        }
    }
}
#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::constant;
    use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
    use crate::laser::gaussian::GaussianBeam;
    use crate::laser::sampler::{LaserSampler, LaserSamplers};
    use crate::magnetic::MagneticFieldSampler;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, Entity, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    fn create_world_for_tests(cooling_light: CoolingLight) -> (World, Entity) {
        let mut test_world = World::new();
        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<AtomInfo>();
        test_world.register::<MagneticFieldSampler>();
        test_world.register::<Force>();
        test_world.register::<LaserSamplers>();

        let e_radius = 2.0;
        let power = 1.0;
        let laser_entity = test_world
            .create_entity()
            .with(cooling_light)
            .with(CoolingLightIndex {
                index: 0,
                initiated: true,
            })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: e_radius,
                power: power,
            })
            .build();
        (test_world, laser_entity)
    }

    #[test]
    fn test_calculate_cooling_force_system() {
        let detuning = 0.0;
        let intensity = 1.0;
        let cooling = CoolingLight::for_species(AtomInfo::rubidium(), detuning, 1.0);
        let wavenumber = cooling.wavenumber();
        let (mut test_world, laser) = create_world_for_tests(cooling);
        test_world.register::<Dark>();
        let atom1 = test_world
            .create_entity()
            .with(Force::new())
            .with(LaserSamplers {
                contents: vec![LaserSampler {
                    scattering_rate: 0.0,
                    force: Vector3::new(0.0, 0.0, 0.0),
                    polarization: 1.0,
                    wavevector: wavenumber * Vector3::new(1.0, 0.0, 0.0),
                    intensity: intensity,
                    doppler_shift: 0.0,
                }],
            })
            .with(MagneticFieldSampler {
                field: Vector3::new(1e-8, 0.0, 0.0),
                magnitude: 1e-8,
            })
            .with(AtomInfo::rubidium())
            .build();

        let mut system = CalculateCoolingForcesSystem {};
        system.run_now(&test_world.res);
        test_world.maintain();

        // See eg Foot, Atomic Physics, p180.
        let cooling_light_storage = test_world.read_storage::<CoolingLight>();
        let cooling_light = cooling_light_storage.get(laser).expect("entity not found");
        let photon_momentum = constant::HBAR * cooling_light.wavenumber();
        let i_norm = intensity / AtomInfo::rubidium().saturation_intensity;
        let scattering_rate = (AtomInfo::rubidium().gamma() / 2.0) * i_norm
            / (1.0 + i_norm + 4.0 * (detuning * 1e6 / AtomInfo::rubidium().linewidth).powf(2.0));
        let f_scatt = photon_momentum * scattering_rate;

        let force_storage = test_world.read_storage::<Force>();
        assert_approx_eq!(
            1e20 * force_storage.get(atom1).expect("entity not found").force[0],
            1e20 * f_scatt,
            1e-6
        );
        assert_eq!(
            force_storage.get(atom1).expect("entity not found").force[1],
            0.0
        );
        assert_eq!(
            force_storage.get(atom1).expect("entity not found").force[2],
            0.0
        );
    }


    #[test]
    fn test_dark() {
        let detuning = 0.0;
        let intensity = 1.0;
        let cooling = CoolingLight::for_species(AtomInfo::rubidium(), detuning, 1.0);
        let wavenumber = cooling.wavenumber();
        let (mut test_world, laser) = create_world_for_tests(cooling);
        test_world.register::<Dark>();
        let atom1 = test_world
            .create_entity()
            .with(Dark {})
            .with(Force::new())
            .with(LaserSamplers {
                contents: vec![LaserSampler {
                    scattering_rate: 0.0,
                    force: Vector3::new(0.0, 0.0, 0.0),
                    polarization: 1.0,
                    wavevector: wavenumber * Vector3::new(1.0, 0.0, 0.0),
                    intensity: intensity,
                    doppler_shift: 0.0,
                }],
            })
            .with(MagneticFieldSampler {
                field: Vector3::new(1e-8, 0.0, 0.0),
                magnitude: 1e-8,
            })
            .with(AtomInfo::rubidium())
            .build();

        let mut system = CalculateCoolingForcesSystem {};
        system.run_now(&test_world.res);
        test_world.maintain();

        let cooling_light_storage = test_world.read_storage::<CoolingLight>();
        let cooling_light = cooling_light_storage.get(laser).expect("entity not found");


        let force_storage = test_world.read_storage::<Force>();
        assert_approx_eq!(
            force_storage.get(atom1).expect("entity not found").force[0],
            0.,
            1e-9
        );
        assert_eq!(
            force_storage.get(atom1).expect("entity not found").force[1],
            0.0
        );
        assert_eq!(
            force_storage.get(atom1).expect("entity not found").force[2],
            0.0
        );
    }
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
            assert_approx_eq!(force[0] as f64, 0.0, 1.0e-30);
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
        test_world.register::<Dark>();
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
