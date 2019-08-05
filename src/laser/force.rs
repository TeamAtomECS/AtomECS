extern crate specs;
use specs::{Join, ReadStorage, System, WriteStorage};

use super::cooling::{CoolingLight, CoolingLightIndex};
use super::gaussian::GaussianBeam;
use super::sampler::LaserSamplers;
use crate::atom::Force;
use crate::constant::{HBAR, PI};
use crate::initiate::AtomInfo;
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
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, AtomInfo>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (laser, laser_indices, beams, magnetic_samplers, laser_samplers, atom_info, mut forces): Self::SystemData,
    ) {
        // Outer loop over atoms
        for (atom_info, bfield, laser_sampler, mut force) in
            (&atom_info, &magnetic_samplers, &laser_samplers, &mut forces).join()
        {
            // Inner loop over cooling lasers
            for (laser, laser_index, beam) in (&laser, &laser_indices, &beams).join() {
                let s0 = laser_sampler.contents[laser_index.index].intensity
                    / atom_info.saturation_intensity;
                let detuning = (laser.frequency()
                    - atom_info.frequency
                    - laser_sampler.contents[laser_index.index].doppler_shift)
                    * 2.0
                    * PI;
                let wavevector = beam.direction * laser.wavenumber();
                let costheta = wavevector.normalize().dot(&bfield.field.normalize());
                let gamma = atom_info.gamma();
                let scatter1 = 0.25 * (laser.polarization * costheta + 1.).powf(2.) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (detuning - atom_info.mup / (2.0 * PI * HBAR) * bfield.magnitude)
                            .powf(2.)
                            / gamma.powf(2.));
                let scatter2 = 0.25 * (laser.polarization * costheta - 1.).powf(2.) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (detuning - atom_info.mum / (2.0 * PI * HBAR) * bfield.magnitude)
                            .powf(2.)
                            / gamma.powf(2.));
                let scatter3 = 0.5 * (1. - costheta.powf(2.)) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (detuning - atom_info.muz / (2.0 * PI * HBAR) * bfield.magnitude)
                            .powf(2.)
                            / gamma.powf(2.));
                let cooling_force = wavevector * s0 * HBAR * (scatter1 + scatter2 + scatter3);
                force.force = force.force + cooling_force;
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
    use crate::laser::sampler::{LaserSampler, LaserSamplers};
    use crate::magnetic::MagneticFieldSampler;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, Entity, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    fn create_world_for_tests(cool_light_detuning: f64) -> (World, Entity) {
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
            .with(CoolingLight::for_species(
                AtomInfo::rubidium(),
                cool_light_detuning,
                1.0,
            ))
            .with(CoolingLightIndex { index: 0 })
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
        let (mut test_world, laser) = create_world_for_tests(detuning);

        let intensity = 1.0;
        let atom1 = test_world
            .create_entity()
            .with(Force::new())
            .with(LaserSamplers {
                contents: vec![LaserSampler {
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
        // See eg Foot, Atomic Physics, p180.
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
