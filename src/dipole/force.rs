use crate::laser::intensity_gradient::LaserIntensityGradientSamplers;
use bevy::prelude::*;
extern crate nalgebra;
use crate::atom::Force;
use crate::dipole::DipoleLight;
use crate::dipole::Polarizability;
use crate::laser::index::LaserIndex;
use crate::integrator::AtomECSBatchStrategy;

/// Calculates forces exerted onto the atoms by dipole laser beams.
///
/// It uses the `LaserIntensityGradientSamplers` and the properties of the `DipoleLight`
/// to add the respective amount of force to `Force`

pub fn apply_dipole_force_system<const N: usize>(
    mut atom_query: Query<(&Polarizability, &LaserIntensityGradientSamplers<N>, &mut Force)>,
    laser_query: Query<(&DipoleLight, &LaserIndex)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(polarizability, sampler, mut force)| {
            for (_dipole, index) in laser_query.iter(){
                force.force +=
                    polarizability.prefactor * sampler.contents[index.index].gradient;
            }
        });
}


#[cfg(test)]
pub mod tests {
    use super::*;

    extern crate bevy;
    use assert_approx_eq::assert_approx_eq;
    use crate::integrator::AtomECSBatchStrategy;
    use bevy::prelude::*;
    extern crate nalgebra;
    use crate::constant;
    use crate::laser;
    use crate::laser::*;
    use crate::laser::intensity_gradient::sample_gaussian_laser_intensity_gradient;
    use crate::laser::gaussian::GaussianBeam;
    use crate::laser::DEFAULT_BEAM_LIMIT;
    use crate::laser::intensity_gradient::LaserIntensityGradientSampler;
    use nalgebra::Vector3;
    use crate::atom::Force;
    use crate::atom::Position;

    #[test]
    fn test_apply_dipole_force_system() {
        let mut test = App::new();
        test.insert_resource(AtomECSBatchStrategy::default());

        let transition_linewidth = 32e6;
        let transition_lambda = 461e-9;

        test
            .world_mut()
            .spawn(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(DipoleLight {
                wavelength: 1064.0e-9,
            });

        let transition =
            Polarizability::calculate_for(1064e-9, transition_lambda, transition_linewidth);

        let atom1 = test
                    .world_mut()
                    .spawn(Force {
                        force: Vector3::new(0.0, 0.0, 0.0),
                    })
                    .insert(LaserIntensityGradientSamplers {
                        contents: [crate::laser::intensity_gradient::LaserIntensityGradientSampler {
                            gradient: Vector3::new(0.0, 1.0, -2.0),
                        }; DEFAULT_BEAM_LIMIT],
                    })
                    .insert(transition)
                    .id();

        test.add_systems(Update, apply_dipole_force_system::<{DEFAULT_BEAM_LIMIT}>);
        test.update();

        let sim_result_force = test.world().entity(atom1).get::<Force>().expect("Entity not found!").force;

        let transition_f = constant::C / transition_lambda;
        let actual_force = 3. * constant::PI * constant::C.powf(2.0)
            / (2. * (2. * constant::PI * transition_f).powf(3.0))
            * transition_linewidth
            * (1. / (transition_f - 1064.0e-9) + 1. / (transition_f + 1064.0e-9))
            * Vector3::new(0.0, 1.0, -2.0);

        assert_approx_eq!(actual_force[0], sim_result_force[0], 1e+8_f64); // Why is the tolerance so massive?
        assert_approx_eq!(actual_force[1], sim_result_force[1], 1e+8_f64);
        assert_approx_eq!(actual_force[2], sim_result_force[2], 1e+8_f64);
    }

    #[test]
    fn test_apply_dipole_force_again_system() {
        let mut test = App::new();

        test.insert_resource(AtomECSBatchStrategy::default());

        let transition_linewidth = 32e6;
        let transition_lambda = 461e-9;

        test.world_mut()
            .spawn(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(DipoleLight {
                wavelength: 1064.0e-9,
            });

        let transition =
            Polarizability::calculate_for(1064e-9, transition_lambda, transition_linewidth);

        let atom1 = test
            .world_mut()
            .spawn(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(LaserIntensityGradientSamplers {
                contents: [crate::laser::intensity_gradient::LaserIntensityGradientSampler {
                    gradient: Vector3::new(-8.4628e+7, -4.33992902e+13, -4.33992902e+13),
                }; crate::laser::DEFAULT_BEAM_LIMIT],
            })
            .insert(transition)
            .id();

        test.add_systems(Update, apply_dipole_force_system::<{DEFAULT_BEAM_LIMIT}>);
        test.update();

        let sim_result_force = test.world().entity(atom1).get::<Force>().expect("Entity not found!").force;

        assert_approx_eq!(-6.386888332902177e-29, sim_result_force[0], 3e-30_f64);
        assert_approx_eq!(-3.11151847e-23, sim_result_force[1], 2e-24_f64);
        assert_approx_eq!(-3.11151847e-23, sim_result_force[2], 2e-24_f64);
    }

    #[test]
    fn test_apply_dipole_force_and_gradient_system() {
        let mut test = App::new();

        test.insert_resource(AtomECSBatchStrategy::default());

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

        test.world_mut()
            .spawn(gaussian_beam)
            .insert(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .insert(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(laser::frame::Frame {
                x_vector: Vector3::y(),
                y_vector: Vector3::z(),
            })
            .insert(RequiresIntensityGradientCalculation);

        let gaussian_beam = GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius,
            power,
            direction: Vector3::y(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&1064.0e-9, &e_radius),
            ellipticity: 0.0,
        };

        test.world_mut()
            .spawn(gaussian_beam)
            .insert(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .insert(LaserIndex {
                index: 1,
                initiated: true,
            })
            .insert(laser::frame::Frame {
                x_vector: Vector3::x(),
                y_vector: Vector3::z(),
            })
            .insert(RequiresIntensityGradientCalculation);

        let transition = Polarizability::calculate_for(1064e-9, 460.7e-9, 32e6);

        let atom1 = test.world_mut()
                        .spawn(crate::atom::Position {
                            pos: Vector3::new(-1.0e-4, -1.0e-4, -2.0e-4),
                        })
                        .insert(Force {
                            force: Vector3::new(0.0, 0.0, 0.0),
                        })
                        .insert(LaserIntensityGradientSamplers {
                            contents: [LaserIntensityGradientSampler::default();
                                DEFAULT_BEAM_LIMIT],
                        })
                        .insert(transition)
                        .id();

        test
        .add_systems(Update, 
                    sample_gaussian_laser_intensity_gradient::<{DEFAULT_BEAM_LIMIT}, RequiresIntensityGradientCalculation>);
        test
        .add_systems(Update, 
                    apply_dipole_force_system::<{DEFAULT_BEAM_LIMIT}>
                        .after(sample_gaussian_laser_intensity_gradient::
                            <{DEFAULT_BEAM_LIMIT}, RequiresIntensityGradientCalculation>));
        test.update();

        let sim_result_force = test.world()
                                    .entity(atom1)
                                    .get::<Force>()
                                    .expect("Entity not found!")
                                    .force;

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
