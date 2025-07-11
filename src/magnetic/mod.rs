//! Magnetic fields and zeeman shift

use bevy::prelude::*;
use nalgebra::{Matrix3, Vector3};

use crate::{initiate::NewlyCreated, integrator::AtomECSBatchStrategy};

pub mod analytic;
pub mod force;
pub mod grid;
pub mod quadrupole;
pub mod top;
pub mod uniform;
use std::fmt;

/// A component that stores the magnetic field at an entity's location.
#[derive(Copy, Clone, Component)]
pub struct MagneticFieldSampler {
    /// Vector representing the magnetic field components along x,y,z in units of Tesla.
    pub field: Vector3<f64>,

    /// Magnitude of the magnetic field in units of Tesla
    pub magnitude: f64,

    /// Local gradient of the magnitude of the magnetic field in T/m
    pub gradient: Vector3<f64>,

    ///Local jacobian of magnetic field
    pub jacobian: Matrix3<f64>,
}
impl MagneticFieldSampler {
    pub fn tesla(b_field: Vector3<f64>) -> Self {
        MagneticFieldSampler {
            field: b_field,
            magnitude: b_field.norm(),
            gradient: Vector3::new(0.0, 0.0, 0.0),
            jacobian: Matrix3::zeros(),
        }
    }
}
impl fmt::Display for MagneticFieldSampler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?},{:?},{:?})",
            self.field[0], self.field[1], self.field[2]
        )
    }
}

impl Default for MagneticFieldSampler {
    fn default() -> Self {
        MagneticFieldSampler {
            field: Vector3::new(0.0, 0.0, 0.0),
            magnitude: 0.0,
            gradient: Vector3::new(0.0, 0.0, 0.0),
            jacobian: Matrix3::zeros(),
        }
    }
}

/// System that clears the magnetic field samplers each frame.
fn clear_magnetic_field_sampler(
    mut query: Query<&mut MagneticFieldSampler>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|mut sampler| {
            sampler.magnitude = 0.;
            sampler.field = Vector3::new(0.0, 0.0, 0.0);
            sampler.gradient = Vector3::new(0.0, 0.0, 0.0);
            sampler.jacobian = Matrix3::zeros();
        });
}

/// System that calculates the magnitude of the magnetic field.
///
/// The magnetic field magnitude is frequently used, so it makes sense to calculate it once and cache the result.
/// This system runs after all other magnetic field systems.
pub fn calculate_magnetic_field_magnitude(
    mut query: Query<&mut MagneticFieldSampler>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|mut sampler| {
            sampler.magnitude = sampler.field.norm();
            if sampler.magnitude.is_nan() {
                sampler.magnitude = 0.0;
            }
        });
}

/// Calculates the gradient of the magnitude of the magnetic field.
fn calculate_magnetic_field_magnitude_gradient(
    mut query: Query<&mut MagneticFieldSampler>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|mut sampler| {
            let mut gradient = Vector3::new(0.0, 0.0, 0.0);
            for i in 0..3 {
                gradient[i] =
                    (1.0 / (sampler.magnitude)) * (sampler.field.dot(&sampler.jacobian.column(i)));
            }
            sampler.gradient = gradient;
        });
}

/// Attachs the MagneticFieldSampler component to newly created atoms.
/// This allows other magnetic Systems to interact with the atom, eg to calculate fields at their location.
fn attach_field_samplers_to_new_atoms(
    query: Query<Entity, (With<NewlyCreated>, Without<MagneticFieldSampler>)>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(MagneticFieldSampler::default());
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemSet)]
pub enum MagneticSystemsSet {
    Set,
    SampleFields,
}

/// A plugin responsible for calculating magnetic fields.
///
/// See the [crate::magnetics] module for more information.
pub struct MagneticsPlugin;
impl Plugin for MagneticsPlugin {
    fn build(&self, app: &mut App) {
        //add_magnetics_systems_to_dispatch(&mut builder.dispatcher_builder, &[]);

        app.add_systems(Update,
            (
                clear_magnetic_field_sampler.before(MagneticSystemsSet::SampleFields),
                analytic::calculate_field_contributions::<quadrupole::QuadrupoleField3D>
                    .in_set(MagneticSystemsSet::SampleFields),
                analytic::calculate_field_contributions::<quadrupole::QuadrupoleField2D>
                    .in_set(MagneticSystemsSet::SampleFields),
                analytic::calculate_field_contributions::<uniform::UniformMagneticField>
                    .in_set(MagneticSystemsSet::SampleFields),
                top::rotate_uniform_fields.in_set(MagneticSystemsSet::SampleFields),
                grid::sample_magnetic_grids.in_set(MagneticSystemsSet::SampleFields),
                calculate_magnetic_field_magnitude.after(MagneticSystemsSet::SampleFields),
                calculate_magnetic_field_magnitude_gradient
                    .after(calculate_magnetic_field_magnitude),
                force::apply_magnetic_forces.after(calculate_magnetic_field_magnitude_gradient),
                attach_field_samplers_to_new_atoms,
            )
                .in_set(MagneticSystemsSet::Set),
        );
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        atom::Position,
        integrator::{AtomECSBatchStrategy, Step, Timestep},
    };

    #[test]
    fn test_magnetics_plugin() {
        let mut app = App::new();
        app.add_plugins(MagneticsPlugin);
        app.insert_resource(AtomECSBatchStrategy::default());
        app.insert_resource(Timestep::default());
        app.insert_resource(Step::default());
        //test_world.insert(crate::integrator::Step { n: 0 });
        //test_world.insert(crate::integrator::Timestep { delta: 1.0e-6 });

        app.world_mut()
            .spawn(uniform::UniformMagneticField {
                field: Vector3::new(2.0, 0.0, 0.0),
            })
            .insert(quadrupole::QuadrupoleField3D::gauss_per_cm(
                100.0,
                Vector3::z(),
            ))
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            });

        let test_entity = app
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(1.0, 1.0, 1.0),
            })
            .insert(MagneticFieldSampler::default())
            .id();

        app.update();

        let sampler = app
            .world()
            .entity(test_entity)
            .get::<MagneticFieldSampler>()
            .expect("Cannot find entity");
        assert_eq!(sampler.field, Vector3::new(2.0 + 1.0, 1.0, -2.0));
    }

    /// Tests that magnetic field samplers are added to newly created atoms.
    #[test]
    fn test_field_samplers_are_added() {
        let mut app = App::new();
        app.insert_resource(Step { n: 0 });
        app.insert_resource(AtomECSBatchStrategy::default());
        app.insert_resource(Timestep { delta: 1.0e-6 });
        app.add_plugins(MagneticsPlugin);
        let sampler_entity = app.world_mut().spawn(NewlyCreated).id();
        app.update();
        assert!(app
            .world()
            .entity(sampler_entity)
            .contains::<MagneticFieldSampler>());
    }

    // Test correct calculation of magnetic field gradient
    #[test]

    fn test_magnetic_gradient_system() {
        let mut app = App::new();
        app.insert_resource(Step { n: 0 });
        app.insert_resource(AtomECSBatchStrategy::default());
        app.insert_resource(Timestep { delta: 1.0e-6 });

        let atom1 = app
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(2.0, 1.0, -5.0),
            })
            .insert(MagneticFieldSampler::default())
            .id();

        app.world_mut()
            .spawn(quadrupole::QuadrupoleField3D::gauss_per_cm(
                2.0,
                Vector3::z(),
            ))
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            });

        app.world_mut()
            .spawn(quadrupole::QuadrupoleField3D::gauss_per_cm(
                1.0,
                Vector3::z(),
            ))
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            });

        app.add_plugins(MagneticsPlugin);
        app.update();

        let test_gradient = app
            .world()
            .entity(atom1)
            .get::<MagneticFieldSampler>()
            .expect("entity not found")
            .gradient;

        assert_approx_eq::assert_approx_eq!(test_gradient[0], 5.8554e-3, 1e-6_f64);
        assert_approx_eq::assert_approx_eq!(test_gradient[1], 2.9277e-3, 1e-6_f64);
        assert_approx_eq::assert_approx_eq!(test_gradient[2], -0.058554, 1e-6_f64);
    }
}
