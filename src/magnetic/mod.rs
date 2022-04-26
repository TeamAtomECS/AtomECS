//! Magnetic fields and zeeman shift

use bevy::{prelude::*, tasks::ComputeTaskPool};
use nalgebra::{Vector3, Matrix3};

use crate::{initiate::NewlyCreated, integrator::BatchSize};

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
    pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
) {
    query.par_for_each_mut(
        &pool, batch_size.0,
        |mut sampler| {
            sampler.magnitude = 0.;
            sampler.field = Vector3::new(0.0, 0.0, 0.0);
            sampler.gradient = Vector3::new(0.0, 0.0, 0.0);
            sampler.jacobian = Matrix3::zeros();
        }
    );
}

/// System that calculates the magnitude of the magnetic field.
///
/// The magnetic field magnitude is frequently used, so it makes sense to calculate it once and cache the result.
/// This system runs after all other magnetic field systems.
fn calculate_magnetic_field_magnitude(
    mut query: Query<&mut MagneticFieldSampler>,
    pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
) {
    query.par_for_each_mut(
        &pool, batch_size.0,
        |mut sampler| {
            sampler.magnitude = sampler.field.norm();
            if sampler.magnitude.is_nan() {
                sampler.magnitude = 0.0;
            }
        }
    );
}

/// Calculates the gradient of the magnitude of the magnetic field.
fn calculate_magnetic_field_magnitude_gradient(
    mut query: Query<&mut MagneticFieldSampler>,
    pool: Res<ComputeTaskPool>,
    batch_size: Res<BatchSize>,
) {
    query.par_for_each_mut(
        &pool, batch_size.0,
        |mut sampler| {
            let mut gradient = Vector3::new(0.0, 0.0, 0.0);
            for i in 0..3 {
                gradient[i] =
                    (1.0 / (sampler.magnitude)) * (sampler.field.dot(&sampler.jacobian.column(i)));
            }
            sampler.gradient = gradient;
        }
    );
}

/// Attachs the MagneticFieldSampler component to newly created atoms.
/// This allows other magnetic Systems to interact with the atom, eg to calculate fields at their location.
fn attach_field_samplers_to_new_atoms(
    query: Query<Entity, (With<NewlyCreated>, Without<MagneticFieldSampler>)>,
    mut commands: Commands
) {
    for entity in query.iter() {
        commands.entity(entity).insert(MagneticFieldSampler::default());
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum MagneticSystems {
    Group,
    ClearMagneticFieldSamplers,
    Sample3DQuadrupoleFields,
    Sample2DQuadrupoleFields,
    SampleUniformMagneticFields,
    SampleMagneticGrids,
    RotateTOPFields,
    CalculateMagneticFieldMagnitude,
    CalculateMagneticFieldMagnitudeGradient,
    ApplyMagneticForces,
    AttachFieldSamplersToNewAtoms
}

/// A plugin responsible for calculating magnetic fields.
///
/// See the [crate::magnetics] module for more information.
pub struct MagneticsPlugin;
impl Plugin for MagneticsPlugin {
    fn build(&self, app: &mut App) {
        //add_magnetics_systems_to_dispatch(&mut builder.dispatcher_builder, &[]);
        app.add_system_set(
            SystemSet::new().label(MagneticSystems::Group)
            .with_system(clear_magnetic_field_sampler.label(MagneticSystems::ClearMagneticFieldSamplers))
            .with_system(analytic::calculate_field_contributions::<quadrupole::QuadrupoleField3D>.label(MagneticSystems::Sample3DQuadrupoleFields).after(MagneticSystems::ClearMagneticFieldSamplers))
            .with_system(analytic::calculate_field_contributions::<quadrupole::QuadrupoleField2D>.label(MagneticSystems::Sample2DQuadrupoleFields).after(MagneticSystems::Sample3DQuadrupoleFields))
            .with_system(analytic::calculate_field_contributions::<uniform::UniformMagneticField>.label(MagneticSystems::SampleUniformMagneticFields).after(MagneticSystems::Sample2DQuadrupoleFields))
            .with_system(top::rotate_uniform_fields.label(MagneticSystems::RotateTOPFields))
            .with_system(grid::sample_magnetic_grids.label(MagneticSystems::SampleMagneticGrids).after(MagneticSystems::SampleUniformMagneticFields))
            .with_system(calculate_magnetic_field_magnitude.label(MagneticSystems::CalculateMagneticFieldMagnitude).after(MagneticSystems::SampleMagneticGrids))
            .with_system(calculate_magnetic_field_magnitude_gradient.label(MagneticSystems::CalculateMagneticFieldMagnitudeGradient).after(MagneticSystems::CalculateMagneticFieldMagnitude))
            .with_system(force::apply_magnetic_forces.label(MagneticSystems::ApplyMagneticForces).after(MagneticSystems::CalculateMagneticFieldMagnitudeGradient))
            .with_system(attach_field_samplers_to_new_atoms.label(MagneticSystems::AttachFieldSamplersToNewAtoms))
        );
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::atom::Position;
    use crate::magnetic::quadrupole::QuadrupoleField3D;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_magnetics_plugin() {
        let mut app = App::new();
        app.add_plugin(MagneticsPlugin);
        app.insert_resource(BatchSize::default());
        app.insert_resource(crate::integrator::Timestep::default());
        app.insert_resource(crate::integrator::Step::default());
        //test_world.insert(crate::integrator::Step { n: 0 });
        //test_world.insert(crate::integrator::Timestep { delta: 1.0e-6 });

        app.world.spawn()
            .insert(uniform::UniformMagneticField {
                field: Vector3::new(2.0, 0.0, 0.0),
            })
            .insert(quadrupole::QuadrupoleField3D::gauss_per_cm(
                100.0,
                Vector3::z(),
            ))
            .insert(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            });

        let test_entity = app.world.spawn()
            .insert(Position {
                pos: Vector3::new(1.0, 1.0, 1.0),
            })
            .insert(MagneticFieldSampler::default())
            .id();

        app.update();

        let sampler = app.world.entity(test_entity).get::<MagneticFieldSampler>().expect("Cannot find entity");
        assert_eq!(
            sampler.field,
            Vector3::new(2.0 + 1.0, 1.0, -2.0)
        );
    }

    // /// Tests that magnetic field samplers are added to newly created atoms.
    // #[test]
    // fn test_field_samplers_are_added() {
    //     let mut test_world = World::new();
    //     register_magnetics_components(&mut test_world);
    //     test_world.register::<NewlyCreated>();
    //     let mut builder = DispatcherBuilder::new();
    //     builder.add(
    //         crate::integrator::VelocityVerletIntegratePositionSystem {},
    //         crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME,
    //         &[],
    //     );
    //     add_magnetics_systems_to_dispatch(&mut builder, &[]);
    //     let mut dispatcher = builder.build();
    //     dispatcher.setup(&mut test_world);
    //     test_world.insert(crate::integrator::Step { n: 0 });
    //     test_world.insert(crate::integrator::Timestep { delta: 1.0e-6 });

    //     let sampler_entity = test_world.create_entity().with(NewlyCreated).build();

    //     dispatcher.dispatch(&test_world);
    //     test_world.maintain();

    //     let samplers = test_world.read_storage::<MagneticFieldSampler>();
    //     assert!(samplers.contains(sampler_entity));
    // }

    // // Test correct calculation of magnetic field gradient
    // #[test]

    // fn test_magnetic_gradient_system() {
    //     let mut test_world = World::new();
    //     register_magnetics_components(&mut test_world);
    //     register_magnetic_trap_components(&mut test_world);
    //     test_world.register::<Position>();

    //     let atom1 = test_world
    //         .create_entity()
    //         .with(Position {
    //             pos: Vector3::new(2.0, 1.0, -5.0),
    //         })
    //         .with(MagneticFieldSampler::default())
    //         .build();

    //     test_world
    //         .create_entity()
    //         .with(QuadrupoleField3D::gauss_per_cm(2.0, Vector3::z()))
    //         .with(Position {
    //             pos: Vector3::new(0.0, 0.0, 0.0),
    //         })
    //         .build();

    //     test_world
    //         .create_entity()
    //         .with(QuadrupoleField3D::gauss_per_cm(1.0, Vector3::z()))
    //         .with(Position {
    //             pos: Vector3::new(0.0, 0.0, 0.0),
    //         })
    //         .build();

    //     let mut quad_system = Sample3DQuadrupoleFieldSystem;
    //     quad_system.run_now(&test_world);

    //     let mut magnitude_system = CalculateMagneticFieldMagnitudeSystem;
    //     magnitude_system.run_now(&test_world);
    //     let mut gradient_system = CalculateMagneticMagnitudeGradientSystem;
    //     gradient_system.run_now(&test_world);

    //     test_world.maintain();
    //     let sampler_storage = test_world.read_storage::<MagneticFieldSampler>();

    //     let test_gradient = sampler_storage
    //         .get(atom1)
    //         .expect("entity not found")
    //         .gradient;

    //     assert_approx_eq!(test_gradient[0], 5.8554e-3, 1e-6_f64);
    //     assert_approx_eq!(test_gradient[1], 2.9277e-3, 1e-6_f64);
    //     assert_approx_eq!(test_gradient[2], -0.058554, 1e-6_f64);
    // }
}
