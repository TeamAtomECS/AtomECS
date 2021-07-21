//! A module to calculate laser beam intensity gradients.
//!
//! Gradients are currently only calculated for beams marked as [DipoleLight](DipoleLight.struct.html).

use specs::prelude::*;

use crate::atom::Position;
use crate::laser::dipole_beam::DipoleLight;
use crate::laser::frame::Frame;
use crate::laser::gaussian::{get_gaussian_beam_intensity_gradient, GaussianBeam};
use crate::laser::index::LaserIndex;
use nalgebra::Vector3;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

/// Represents the laser intensity at the position of the atom with respect to a certain laser beam
#[derive(Clone, Copy)]
pub struct LaserIntensityGradientSampler {
    /// Intensity in SI units of W/m^2
    pub gradient: Vector3<f64>,
}

impl Default for LaserIntensityGradientSampler {
    fn default() -> Self {
        LaserIntensityGradientSampler {
            /// Intensity in SI units of W/m^2
            gradient: Vector3::new(f64::NAN, f64::NAN, f64::NAN),
        }
    }
}

/// Component that holds a list of `LaserIntensityGradientSampler`s
pub struct LaserIntensityGradientSamplers {
    /// List of laser gradient samplers
    pub contents: [LaserIntensityGradientSampler; crate::laser::BEAM_LIMIT],
}
impl Component for LaserIntensityGradientSamplers {
    type Storage = VecStorage<Self>;
}

/// Calculates the intensity gradient of each laser beam. The result is stored in the `LaserIntensityGradientSamplers` .
///
/// So far, the only intensity distribution implemented is `GaussianBeam`. Additionally
/// the system also uses `GaussianRayleighRange` for axial divergence and
/// `Frame` to account for different ellipiticies in the future.
/// The result is stored in the `LaserIntensityGradientSamplers` component that each
/// atom is associated with.
pub struct SampleGaussianLaserIntensityGradientSystem;
impl<'a> System<'a> for SampleGaussianLaserIntensityGradientSystem {
    type SystemData = (
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, LaserIndex>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, Frame>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, LaserIntensityGradientSamplers>,
    );

    fn run(
        &mut self,
        (dipole, index, gaussian, reference_frame, pos, mut sampler): Self::SystemData,
    ) {
        use rayon::prelude::*;

        for (_dipole, index, beam, reference) in
            (&dipole, &index, &gaussian, &reference_frame).join()
        {
            (&pos, &mut sampler).par_join().for_each(|(pos, sampler)| {
                sampler.contents[index.index].gradient =
                    get_gaussian_beam_intensity_gradient(beam, pos, reference);
            });
        }
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
    fn test_sample_laser_intensity_gradient_system() {
        let mut test_world = World::new();

        test_world.register::<LaserIndex>();
        test_world.register::<GaussianBeam>();
        test_world.register::<Position>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<Frame>();

        let beam = GaussianBeam {
            direction: Vector3::z(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 70.71067812e-6,
            power: 100.0,
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &1064.0e-9,
                &70.71067812e-6,
            ),
            ellipticity: 0.0,
        };

        test_world
            .create_entity()
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(beam)
            .with(Frame {
                x_vector: Vector3::x(),
                y_vector: Vector3::y(),
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [LaserIntensityGradientSampler::default(); crate::laser::BEAM_LIMIT],
            })
            .build();
        let mut system = SampleGaussianLaserIntensityGradientSystem;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserIntensityGradientSamplers>();
        let sim_result_gradient = sampler_storage
            .get(atom1)
            .expect("Entity not found!")
            .contents[0]
            .gradient;

        let actual_intensity_gradient =
            crate::laser::gaussian::get_gaussian_beam_intensity_gradient(
                &beam,
                &Position {
                    pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
                },
                &Frame {
                    x_vector: Vector3::x(),
                    y_vector: Vector3::y(),
                },
            );

        assert_approx_eq!(
            actual_intensity_gradient[0],
            sim_result_gradient[0],
            1e+5_f64
        );
        assert_approx_eq!(
            actual_intensity_gradient[1],
            sim_result_gradient[1],
            1e+5_f64
        );
        assert_approx_eq!(
            actual_intensity_gradient[2],
            sim_result_gradient[2],
            1e+5_f64
        );
    }
    #[test]
    fn test_sample_laser_intensity_gradient_again_system() {
        let mut test_world = World::new();

        test_world.register::<LaserIndex>();
        test_world.register::<GaussianBeam>();
        test_world.register::<Position>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<Frame>();

        let beam = GaussianBeam {
            direction: Vector3::x(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 70.71067812e-6,
            power: 100.0,
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &1064.0e-9,
                &70.71067812e-6,
            ),
            ellipticity: 0.0,
        };

        test_world
            .create_entity()
            .with(LaserIndex {
                index: 0,
                initiated: true,
            })
            .with(beam)
            .with(Frame {
                x_vector: Vector3::y(),
                y_vector: Vector3::z(),
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(20.0e-6, 20.0e-6, 20.0e-6),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [LaserIntensityGradientSampler::default(); crate::laser::BEAM_LIMIT],
            })
            .build();
        let mut system = SampleGaussianLaserIntensityGradientSystem;
        system.run_now(&test_world);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserIntensityGradientSamplers>();
        let sim_result_gradient = sampler_storage
            .get(atom1)
            .expect("Entity not found!")
            .contents[0]
            .gradient;

        assert_approx_eq!(-8.4628e+7, sim_result_gradient[0], 1e+5_f64);
        assert_approx_eq!(-4.33992902e+13, sim_result_gradient[1], 1e+8_f64);
        assert_approx_eq!(-4.33992902e+13, sim_result_gradient[2], 1e+8_f64);
    }
}
