extern crate specs;
use crate::atom::{Atom, AtomInfo};
use crate::constant;
use rand::Rng;
use specs::{Component, HashMapStorage, Join, ReadExpect, ReadStorage, System, WriteStorage};
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
                //let s0 = 1.0;
                let s0 = laser_sampler.intensity / atom_info.saturation_intensity;
                //println!("s0 : {}", s0);
                let angular_detuning = (laser_sampler.wavevector.norm() * constant::C / 2. / PI
                    - atom_info.frequency)
                    * 2.0
                    * PI
                    - laser_sampler.doppler_shift;
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

pub struct RandomWalkMarker;

impl Component for RandomWalkMarker {
    type Storage = HashMapStorage<Self>;
}

pub struct RandomWalkSystem;

impl<'a> System<'a> for RandomWalkSystem {
    type SystemData = (
        ReadStorage<'a, RandomWalkMarker>,
        WriteStorage<'a, Force>,
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomInfo>,
        ReadExpect<'a, Timestep>,
    );

    fn run(&mut self, (_rand, mut force, samplers, _atom, atom_info, timestep): Self::SystemData) {
        for _ in (&_rand).join() {
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
                println!("collision{}", number_collision);
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
            break;
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
        let atom1 = test_world
            .create_entity()
            .with(Force::new())
            .with(LaserSamplers {
                contents: vec![LaserSampler {
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
            force_storage.get(atom1).expect("entity not found").force[0],
            f_scatt,
            1e-30
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
}
