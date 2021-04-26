//! Calculation of the intensity of CoolingLight entities at a specific position
//!

// This file exists because - in the spirit of keeping things general - I thought that the intensity sampler should not be in
// gaussian.rs since other beam profiles (although they're less common) should not be excluded.

extern crate rayon;
extern crate specs;

use crate::atom::Position;
use crate::dipole::dipole_beam::DipoleLightIndex;
use crate::laser::gaussian::{
    get_gaussian_beam_intensity_gradient, GaussianBeam, GaussianReferenceFrame,
};
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
    pub contents: [LaserIntensityGradientSampler; crate::dipole::DIPOLE_BEAM_LIMIT],
}
impl Component for LaserIntensityGradientSamplers {
    type Storage = VecStorage<Self>;
}

/// System that calculates the intensity gradient of entities that represent
/// a dipole laser beam
///
/// So far, the only intensity distribution implemented is`GaussianBeam`. Additionally
/// the system also uses `GaussianRayleighRange` for axial divergence and
/// `GaussianReferenceFrame` to account for different ellipiticies in the future.
/// The result is stored in the `LaserIntensityGradientSamplers` component that each
/// atom is associated with.
pub struct SampleLaserIntensityGradientSystem;
impl<'a> System<'a> for SampleLaserIntensityGradientSystem {
    type SystemData = (
        ReadStorage<'a, DipoleLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, GaussianReferenceFrame>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, LaserIntensityGradientSamplers>,
    );

    fn run(
        &mut self,
        (dipole_index, gaussian, reference_frame, pos, mut sampler): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (index, beam, reference) in (&dipole_index, &gaussian, &reference_frame).join() {
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

        test_world.register::<DipoleLightIndex>();
        test_world.register::<GaussianBeam>();
        test_world.register::<Position>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<GaussianReferenceFrame>();

        let beam = GaussianBeam {
            direction: Vector3::z(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 70.71067812e-6,
            power: 100.0,
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &1064.0e-9,
                &70.71067812e-6,
            ),
        };

        test_world
            .create_entity()
            .with(DipoleLightIndex {
                index: 0,
                initiated: true,
            })
            .with(beam)
            .with(GaussianReferenceFrame {
                x_vector: Vector3::x(),
                y_vector: Vector3::y(),
                ellipticity: 0.0,
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [LaserIntensityGradientSampler::default();
                    crate::dipole::DIPOLE_BEAM_LIMIT],
            })
            .build();
        let mut system = SampleLaserIntensityGradientSystem;
        system.run_now(&test_world.res);
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
                &GaussianReferenceFrame {
                    x_vector: Vector3::x(),
                    y_vector: Vector3::y(),
                    ellipticity: 0.0,
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

        test_world.register::<DipoleLightIndex>();
        test_world.register::<GaussianBeam>();
        test_world.register::<Position>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<GaussianReferenceFrame>();

        let beam = GaussianBeam {
            direction: Vector3::x(),
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 70.71067812e-6,
            power: 100.0,
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &1064.0e-9,
                &70.71067812e-6,
            ),
        };

        test_world
            .create_entity()
            .with(DipoleLightIndex {
                index: 0,
                initiated: true,
            })
            .with(beam)
            .with(GaussianReferenceFrame {
                x_vector: Vector3::y(),
                y_vector: Vector3::z(),
                ellipticity: 0.0,
            })
            .build();

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(20.0e-6, 20.0e-6, 20.0e-6),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [LaserIntensityGradientSampler::default();
                    crate::dipole::DIPOLE_BEAM_LIMIT],
            })
            .build();
        let mut system = SampleLaserIntensityGradientSystem;
        system.run_now(&test_world.res);
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
