extern crate rayon;
extern crate specs;
use crate::constant;
use crate::dipole::intensity_gradient::LaserIntensityGradientSamplers;
use specs::{Join, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use crate::atom::Force;
use crate::dipole::atom::AtomicDipoleTransition;
use crate::dipole::dipole_beam::{DipoleLight, DipoleLightIndex};
use nalgebra::Vector3;

/// System that calculates the forces exerted onto the atoms by the dipole laser beams
/// It uses the `LaserIntensityGradientSamplers` and the properties of the `DipoleLight`
/// to add the respective amount of force to `Force`
pub struct ApplyDipoleForceSystem;

impl<'a> System<'a> for ApplyDipoleForceSystem {
    type SystemData = (
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, DipoleLightIndex>,
        ReadStorage<'a, AtomicDipoleTransition>,
        ReadStorage<'a, LaserIntensityGradientSamplers>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (dipole_light, dipole_index,atomic_transition, gradient_sampler, mut force): Self::SystemData,
    ) {
        use rayon::prelude::ParallelIterator;
        use specs::ParJoin;
        (&mut force, &atomic_transition, &gradient_sampler)
            .par_join()
            .for_each(|(mut force, atominfo, sampler)| {
                let prefactor = -3. * constant::PI * constant::C.powf(2.0)
                    / (2. * (2. * constant::PI * atominfo.frequency).powf(3.0))
                    * atominfo.linewidth;
                let mut temp_force_coeff = Vector3::new(0.0, 0.0, 0.0);
                for (index, dipole) in (&dipole_index, &dipole_light).join() {
                    temp_force_coeff = temp_force_coeff
                        - (1. / (atominfo.frequency - dipole.frequency())
                            + 1. / (atominfo.frequency + dipole.frequency()))
                            * sampler.contents[index.index].gradient;
                }
                force.force = force.force + prefactor * temp_force_coeff;
            });
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    #[test]
    fn test_apply_dipole_force_system() {
        let mut test_world = World::new();

        test_world.register::<DipoleLightIndex>();
        test_world.register::<DipoleLight>();
        test_world.register::<Force>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<AtomicDipoleTransition>();

        test_world
            .create_entity()
            .with(DipoleLightIndex {
                index: 0,
                initiated: true,
            })
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .build();

        let transition = AtomicDipoleTransition::strontium();
        let atom1 = test_world
            .create_entity()
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [crate::dipole::intensity_gradient::LaserIntensityGradientSampler {
                    gradient: Vector3::new(0.0, 1.0, -2.0),
                }; crate::dipole::DIPOLE_BEAM_LIMIT],
            })
            .with(transition)
            .build();
        let mut system = ApplyDipoleForceSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();
        let sim_result_force = sampler_storage.get(atom1).expect("Entity not found!").force;

        let actual_force = 3. * constant::PI * constant::C.powf(2.0)
            / (2. * (2. * constant::PI * transition.frequency).powf(3.0))
            * transition.linewidth
            * (1. / (transition.frequency - 1064.0e-9) + 1. / (transition.frequency + 1064.0e-9))
            * Vector3::new(0.0, 1.0, -2.0);

        assert_approx_eq!(actual_force[0], sim_result_force[0], 1e+8_f64);
        assert_approx_eq!(actual_force[1], sim_result_force[1], 1e+8_f64);
        assert_approx_eq!(actual_force[2], sim_result_force[2], 1e+8_f64);
    }

    #[test]
    fn test_apply_dipole_force_again_system() {
        let mut test_world = World::new();

        test_world.register::<DipoleLightIndex>();
        test_world.register::<DipoleLight>();
        test_world.register::<Force>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<AtomicDipoleTransition>();

        test_world
            .create_entity()
            .with(DipoleLightIndex {
                index: 0,
                initiated: true,
            })
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .build();

        let transition = AtomicDipoleTransition::strontium();
        let atom1 = test_world
            .create_entity()
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [crate::dipole::intensity_gradient::LaserIntensityGradientSampler {
                    gradient: Vector3::new(-8.4628e+7, -4.33992902e+13, -4.33992902e+13),
                }; crate::dipole::DIPOLE_BEAM_LIMIT],
            })
            .with(transition)
            .build();
        let mut system = ApplyDipoleForceSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();
        let sim_result_force = sampler_storage.get(atom1).expect("Entity not found!").force;

        assert_approx_eq!(-6.06743188e-29, sim_result_force[0], 3e-30_f64);
        assert_approx_eq!(-3.11151847e-23, sim_result_force[1], 2e-24_f64);
        assert_approx_eq!(-3.11151847e-23, sim_result_force[2], 2e-24_f64);
    }
}
