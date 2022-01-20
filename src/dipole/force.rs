use crate::laser::intensity_gradient::LaserIntensityGradientSamplers;
use specs::prelude::*;
use specs::{Join, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use crate::atom::Force;
use crate::dipole::DipoleLight;
use crate::dipole::Polarizability;
use crate::laser::index::LaserIndex;

/// Calculates forces exerted onto the atoms by dipole laser beams.
///
/// It uses the `LaserIntensityGradientSamplers` and the properties of the `DipoleLight`
/// to add the respective amount of force to `Force`
pub struct ApplyDipoleForceSystem<const N: usize>;

impl<'a, const N: usize> System<'a> for ApplyDipoleForceSystem<N> {
    type SystemData = (
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, LaserIndex>,
        ReadStorage<'a, Polarizability>,
        ReadStorage<'a, LaserIntensityGradientSamplers<N>>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (dipole_light, dipole_index, polarizability, gradient_sampler, mut force): Self::SystemData,
    ) {
        (&mut force, &polarizability, &gradient_sampler)
            .par_join()
            .for_each(|(force, polarizability, sampler)| {
                for (index, _dipole) in (&dipole_index, &dipole_light).join() {
                    force.force +=
                        polarizability.prefactor * sampler.contents[index.index].gradient;
                }
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
    use crate::constant;
    use crate::laser;
    use crate::laser::gaussian::GaussianBeam;
    use crate::laser::DEFAULT_BEAM_LIMIT;
    use nalgebra::Vector3;

    #[test]
    fn test_apply_dipole_force_system() {
        let mut test_world = World::new();

        test_world.register::<LaserIndex>();
        test_world.register::<DipoleLight>();
        test_world.register::<Force>();
        test_world.register::<LaserIntensityGradientSamplers<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Polarizability>();

        let transition_linewidth = 32e6;
        let transition_lambda = 461e-9;
        test_world
            .create_entity()
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .build();

        let transition =
            Polarizability::calculate_for(1064e-9, transition_lambda, transition_linewidth);
        let atom1 = test_world
            .create_entity()
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [crate::laser::intensity_gradient::LaserIntensityGradientSampler {
                    gradient: Vector3::new(0.0, 1.0, -2.0),
                }; crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(transition)
            .build();
        let mut system = ApplyDipoleForceSystem::<{ DEFAULT_BEAM_LIMIT }>;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();
        let sim_result_force = sampler_storage.get(atom1).expect("Entity not found!").force;

        let transition_f = constant::C / transition_lambda;
        let actual_force = 3. * constant::PI * constant::C.powf(2.0)
            / (2. * (2. * constant::PI * transition_f).powf(3.0))
            * transition_linewidth
            * (1. / (transition_f - 1064.0e-9) + 1. / (transition_f + 1064.0e-9))
            * Vector3::new(0.0, 1.0, -2.0);

        assert_approx_eq!(actual_force[0], sim_result_force[0], 1e+8_f64);
        assert_approx_eq!(actual_force[1], sim_result_force[1], 1e+8_f64);
        assert_approx_eq!(actual_force[2], sim_result_force[2], 1e+8_f64);
    }

    #[test]
    fn test_apply_dipole_force_again_system() {
        let mut test_world = World::new();

        test_world.register::<LaserIndex>();
        test_world.register::<DipoleLight>();
        test_world.register::<Force>();
        test_world.register::<LaserIntensityGradientSamplers<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Polarizability>();

        test_world
            .create_entity()
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .build();

        let transition = Polarizability::calculate_for(1064e-9, 461e-9, 32e6);
        let atom1 = test_world
            .create_entity()
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [crate::laser::intensity_gradient::LaserIntensityGradientSampler {
                    gradient: Vector3::new(-8.4628e+7, -4.33992902e+13, -4.33992902e+13),
                }; crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(transition)
            .build();
        let mut system = ApplyDipoleForceSystem::<{ DEFAULT_BEAM_LIMIT }>;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();
        let sim_result_force = sampler_storage.get(atom1).expect("Entity not found!").force;

        assert_approx_eq!(-6.386888332902177e-29, sim_result_force[0], 3e-30_f64);
        assert_approx_eq!(-3.11151847e-23, sim_result_force[1], 2e-24_f64);
        assert_approx_eq!(-3.11151847e-23, sim_result_force[2], 2e-24_f64);
    }

    #[test]
    fn test_apply_dipole_force_and_gradient_system() {
        let mut test_world = World::new();

        test_world.register::<LaserIndex>();
        test_world.register::<DipoleLight>();
        test_world.register::<Force>();
        test_world.register::<LaserIntensityGradientSamplers<{ DEFAULT_BEAM_LIMIT }>>();
        test_world.register::<Polarizability>();
        test_world.register::<crate::atom::Position>();
        test_world.register::<crate::laser::gaussian::GaussianBeam>();
        test_world.register::<crate::laser::frame::Frame>();

        let power = 10.0;
        let e_radius = 60.0e-6 / (2.0_f64.sqrt());

        let gaussian_beam = GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius,
            power,
            direction: Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&1064.0e-9, &e_radius),
            ellipticity: 0.0,
        };
        test_world
            .create_entity()
            .with(gaussian_beam)
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(laser::frame::Frame {
                x_vector: Vector3::y(),
                y_vector: Vector3::z(),
            })
            .build();
        let gaussian_beam = GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius,
            power,
            direction: Vector3::y(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&1064.0e-9, &e_radius),
            ellipticity: 0.0,
        };
        test_world
            .create_entity()
            .with(gaussian_beam)
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .with(LaserIndex {
                index: 1,
                initiated: true,
            })
            .with(laser::frame::Frame {
                x_vector: Vector3::x(),
                y_vector: Vector3::z(),
            })
            .build();

        let transition = Polarizability::calculate_for(1064e-9, 460.7e-9, 32e6);
        let atom1 = test_world
            .create_entity()
            .with(crate::atom::Position {
                pos: Vector3::new(-1.0e-4, -1.0e-4, -2.0e-4),
            })
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [laser::intensity_gradient::LaserIntensityGradientSampler::default();
                    crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .with(transition)
            .build();
        let mut grad_system = laser::intensity_gradient::SampleGaussianLaserIntensityGradientSystem::<
            { DEFAULT_BEAM_LIMIT },
        >;
        let mut force_system = ApplyDipoleForceSystem::<{ DEFAULT_BEAM_LIMIT }>;
        grad_system.run_now(&test_world);
        test_world.maintain();
        force_system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();
        let grad_sampler_storage =
            test_world.read_storage::<LaserIntensityGradientSamplers<{ DEFAULT_BEAM_LIMIT }>>();
        let sim_result_force = sampler_storage.get(atom1).expect("Entity not found!").force;
        let _sim_result_grad = grad_sampler_storage
            .get(atom1)
            .expect("Entity not found!")
            .contents;
        //println!("force is: {}", sim_result_force);
        //println!("gradient 1 is: {}", sim_result_grad[0].gradient);
        //println!("gradient 2 is: {}", sim_result_grad[1].gradient);

        assert_approx_eq!(
            0.000000000000000000000000000000000127913190642808,
            sim_result_force[0],
            3e-46_f64
        );
        assert_approx_eq!(
            0.000000000000000000000000000000000127913190642808,
            sim_result_force[1],
            2e-46_f64
        );
        assert_approx_eq!(
            0.000000000000000000000000000000000511875188257342,
            sim_result_force[2],
            2e-46_f64
        );
    }
}
