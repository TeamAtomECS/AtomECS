//! A module to calculate laser beam intensity gradients.
//!
//! Gradients are currently only calculated for beams marked as [DipoleLight](DipoleLight.struct.html).

use bevy::prelude::*;

use crate::atom::Position;
use crate::integrator::AtomECSBatchStrategy;
use crate::laser::frame::Frame;
use crate::laser::gaussian::{get_gaussian_beam_intensity_gradient, GaussianBeam};
use crate::laser::index::LaserIndex;
use nalgebra::Vector3;

/// Represents the laser intensity at the position of the atom with respect to a certain laser beam
#[derive(Clone, Copy)]
pub struct LaserIntensityGradientSampler {
    /// Intensity in SI units of W/m^2
    pub gradient: Vector3<f64>,
}

impl Default for LaserIntensityGradientSampler {
    fn default() -> Self {
        LaserIntensityGradientSampler {
            // Intensity in SI units of W/m^2
            gradient: Vector3::new(f64::NAN, f64::NAN, f64::NAN),
        }
    }
}

/// Component that holds a list of `LaserIntensityGradientSampler`s
#[derive(Component)]
pub struct LaserIntensityGradientSamplers<const N: usize> {
    /// List of laser gradient samplers
    pub contents: [LaserIntensityGradientSampler; N],
}

/// Calculates the intensity gradient of each laser beam. The result is stored in the `LaserIntensityGradientSamplers` .
///
/// So far, the only intensity distribution implemented is `GaussianBeam`. Additionally
/// the system also uses `GaussianRayleighRange` for axial divergence and
/// `Frame` to account for different ellipiticies in the future.
/// The result is stored in the `LaserIntensityGradientSamplers` component that each
/// atom is associated with.
pub fn sample_gaussian_laser_intensity_gradient<const N: usize, FilterT>(
    laser_query: Query<(&LaserIndex, &GaussianBeam, &Frame), With<FilterT>>,
    mut sampler_query: Query<(&mut LaserIntensityGradientSamplers<N>, &Position)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) where
    FilterT: Component + Send + Sync,
{
    for (index, beam, frame) in laser_query.iter() {
        sampler_query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(mut sampler, pos)| {
                sampler.contents[index.index].gradient =
                    get_gaussian_beam_intensity_gradient(beam, pos, frame);
            });
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;

    #[derive(Component)]
    struct TestComp;

    #[test]
    fn test_sample_laser_intensity_gradient_system() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());

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

        app.world_mut()
            .spawn(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(beam)
            .insert(Frame {
                x_vector: Vector3::x(),
                y_vector: Vector3::y(),
            })
            .insert(TestComp);

        let atom1 = app
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(10.0e-6, 0.0, 30.0e-6),
            })
            .insert(LaserIntensityGradientSamplers {
                contents: [LaserIntensityGradientSampler::default(); 1],
            })
            .id();

        app.add_systems(Update, sample_gaussian_laser_intensity_gradient::<1, TestComp>);
        app.update();

        let sim_result_gradient = app
            .world()
            .entity(atom1)
            .get::<LaserIntensityGradientSamplers<1>>()
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
    fn test_sample_laser_intensity_gradient_numbers() {
        let mut app = App::new();
        app.insert_resource(AtomECSBatchStrategy::default());

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

        app.world_mut()
            .spawn(LaserIndex {
                index: 0,
                initiated: true,
            })
            .insert(beam)
            .insert(Frame {
                x_vector: Vector3::y(),
                y_vector: Vector3::z(),
            })
            .insert(TestComp);

        let atom1 = app
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(20.0e-6, 20.0e-6, 20.0e-6),
            })
            .insert(LaserIntensityGradientSamplers {
                contents: [LaserIntensityGradientSampler::default(); 1],
            })
            .id();

        app.add_systems(Update, sample_gaussian_laser_intensity_gradient::<1, TestComp>);
        app.update();

        let sim_result_gradient = app
            .world()
            .entity(atom1)
            .get::<LaserIntensityGradientSamplers<1>>()
            .expect("Entity not found!")
            .contents[0]
            .gradient;

        assert_approx_eq!(-8.4628e+7, sim_result_gradient[0], 1e+5_f64);
        assert_approx_eq!(-4.33992902e+13, sim_result_gradient[1], 1e+8_f64);
        assert_approx_eq!(-4.33992902e+13, sim_result_gradient[2], 1e+8_f64);
    }
}
