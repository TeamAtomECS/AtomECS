//! Magnetic fields and zeeman shift

extern crate nalgebra;

use specs::prelude::*;

use crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME;
use crate::{initiate::NewlyCreated, simulation::Plugin};
use nalgebra::{Matrix3, Vector3};
use specs::{
    Component, DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadStorage, System,
    VecStorage, World, WriteStorage,
};

pub mod force;
pub mod grid;
pub mod quadrupole;
pub mod top;
pub mod uniform;
use std::fmt;

/// A component that stores the magnetic field at an entity's location.
#[derive(Copy, Clone)]
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
impl Component for MagneticFieldSampler {
    type Storage = VecStorage<Self>;
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
pub struct ClearMagneticFieldSamplerSystem;

impl<'a> System<'a> for ClearMagneticFieldSamplerSystem {
    type SystemData = WriteStorage<'a, MagneticFieldSampler>;
    fn run(&mut self, mut sampler: Self::SystemData) {
        use rayon::prelude::*;

        (&mut sampler).par_join().for_each(|mut sampler| {
            sampler.magnitude = 0.;
            sampler.field = Vector3::new(0.0, 0.0, 0.0);
            sampler.gradient = Vector3::new(0.0, 0.0, 0.0);
            sampler.jacobian = Matrix3::zeros();
        });
    }
}

/// System that calculates the magnitude of the magnetic field.
///
/// The magnetic field magnitude is frequently used, so it makes sense to calculate it once and cache the result.
/// This system runs after all other magnetic field systems.
pub struct CalculateMagneticFieldMagnitudeSystem;

impl<'a> System<'a> for CalculateMagneticFieldMagnitudeSystem {
    type SystemData = WriteStorage<'a, MagneticFieldSampler>;
    fn run(&mut self, mut sampler: Self::SystemData) {
        use rayon::prelude::*;

        (&mut sampler).par_join().for_each(|mut sampler| {
            sampler.magnitude = sampler.field.norm();
            if sampler.magnitude.is_nan() {
                sampler.magnitude = 0.0;
            }
        });
    }
}

/// System that calculates the gradient of the magnitude of the magnetic field.
///

pub struct CalculateMagneticMagnitudeGradientSystem;

impl<'a> System<'a> for CalculateMagneticMagnitudeGradientSystem {
    type SystemData = WriteStorage<'a, MagneticFieldSampler>;
    fn run(&mut self, mut sampler: Self::SystemData) {
        use rayon::prelude::*;

        (&mut sampler).par_join().for_each(|mut sampler| {
            let mut gradient = Vector3::new(0.0, 0.0, 0.0);
            for i in 0..3 {
                gradient[i] =
                    (1.0 / (sampler.magnitude)) * (sampler.field.dot(&sampler.jacobian.column(i)));
            }
            sampler.gradient = gradient;
        });
    }
}

/// Attachs the MagneticFieldSampler component to newly created atoms.
/// This allows other magnetic Systems to interact with the atom, eg to calculate fields at their location.
pub struct AttachFieldSamplersToNewlyCreatedAtomsSystem;

impl<'a> System<'a> for AttachFieldSamplersToNewlyCreatedAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, NewlyCreated>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
        for (ent, _nc) in (&ent, &newly_created).join() {
            updater.insert(ent, MagneticFieldSampler::default());
        }
    }
}

/// Adds the systems required by magnetics to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the magnetics systems run.
fn add_magnetics_systems_to_dispatch(
    builder: &mut DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) {
    builder.add(ClearMagneticFieldSamplerSystem, "magnetics_clear", deps);
    builder.add(
        quadrupole::Sample3DQuadrupoleFieldSystem,
        "magnetics_quadrupole",
        &[
            "magnetics_clear",
            crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME,
        ],
    );
    builder.add(
        quadrupole::Sample2DQuadrupoleFieldSystem,
        "magnetics_2dquadrupole",
        &["magnetics_quadrupole"],
    );
    builder.add(
        uniform::UniformMagneticFieldSystem,
        "magnetics_uniform",
        &["magnetics_2dquadrupole"],
    );
    builder.add(
        top::TimeOrbitingPotentialSystem,
        "magnetics_top",
        &["magnetics_uniform"],
    );
    builder.add(
        grid::SampleMagneticGridSystem,
        "magnetics_grid",
        &["magnetics_top", INTEGRATE_POSITION_SYSTEM_NAME],
    );
    builder.add(
        CalculateMagneticFieldMagnitudeSystem,
        "magnetics_magnitude",
        &["magnetics_grid"],
    );
    builder.add(
        AttachFieldSamplersToNewlyCreatedAtomsSystem,
        "add_magnetic_field_samplers",
        &[],
    );
}

/// Adds the additional systems required by magnetics to the dispatcher.
fn add_magnetic_trap_systems_to_dispatch(builder: &mut DispatcherBuilder<'static, 'static>) {
    builder.add(
        CalculateMagneticMagnitudeGradientSystem,
        "magnetics_gradient",
        &["magnetics_magnitude"],
    );
    builder.add(
        force::ApplyMagneticForceSystem,
        "magnetic_force",
        &["magnetics_gradient"],
    );
}

/// Registers resources required by magnetics to the ecs world.
fn register_magnetics_components(world: &mut World) {
    world.register::<uniform::UniformMagneticField>();
    world.register::<quadrupole::QuadrupoleField3D>();
    world.register::<quadrupole::QuadrupoleField2D>();
    world.register::<top::TimeOrbitingPotential>();
    world.register::<MagneticFieldSampler>();
    world.register::<grid::PrecalculatedMagneticFieldGrid>();
    world.register::<force::MagneticDipole>();
}

/// Registers additional resources required by magnetic trapping to the ecs world.
fn register_magnetic_trap_components(world: &mut World) {
    world.register::<force::MagneticDipole>();
}

/// A plugin responsible for calculating magnetic fields.
///
/// See the [crate::magnetics] module for more information.
pub struct MagneticsPlugin;
impl Plugin for MagneticsPlugin {
    fn build(&self, builder: &mut crate::simulation::SimulationBuilder) {
        add_magnetics_systems_to_dispatch(&mut builder.dispatcher_builder, &[]);
        register_magnetics_components(&mut builder.world);
    }

    fn deps(&self) -> Vec<Box<dyn Plugin>> {
        Vec::new()
    }
}

/// Plugin for magnetic confinement functionality
pub struct MagneticTrapPlugin;
impl Plugin for MagneticTrapPlugin {
    fn build(&self, builder: &mut crate::simulation::SimulationBuilder) {
        register_magnetic_trap_components(&mut builder.world);
        add_magnetic_trap_systems_to_dispatch(&mut builder.dispatcher_builder);
    }

    fn deps(&self) -> Vec<Box<dyn Plugin>> {
        vec![Box::new(MagneticsPlugin)]
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::atom::Position;
    use crate::magnetic::quadrupole::{QuadrupoleField3D, Sample3DQuadrupoleFieldSystem};
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_magnetics_systems() {
        let mut test_world = World::new();
        register_magnetics_components(&mut test_world);
        test_world.register::<NewlyCreated>();
        let mut builder = DispatcherBuilder::new();
        builder.add(
            crate::integrator::VelocityVerletIntegratePositionSystem {},
            crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME,
            &[],
        );
        add_magnetics_systems_to_dispatch(&mut builder, &[]);
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut test_world);
        test_world.insert(crate::integrator::Step { n: 0 });
        test_world.insert(crate::integrator::Timestep { delta: 1.0e-6 });

        test_world
            .create_entity()
            .with(uniform::UniformMagneticField {
                field: Vector3::new(2.0, 0.0, 0.0),
            })
            .with(quadrupole::QuadrupoleField3D::gauss_per_cm(
                100.0,
                Vector3::z(),
            ))
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();

        let sampler_entity = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(1.0, 1.0, 1.0),
            })
            .with(MagneticFieldSampler::default())
            .build();

        dispatcher.dispatch(&test_world);

        let samplers = test_world.read_storage::<MagneticFieldSampler>();
        let sampler = samplers.get(sampler_entity);
        assert_eq!(
            sampler.expect("entity not found").field,
            Vector3::new(2.0 + 1.0, 1.0, -2.0)
        );
    }

    /// Tests that magnetic field samplers are added to newly created atoms.
    #[test]
    fn test_field_samplers_are_added() {
        let mut test_world = World::new();
        register_magnetics_components(&mut test_world);
        test_world.register::<NewlyCreated>();
        let mut builder = DispatcherBuilder::new();
        builder.add(
            crate::integrator::VelocityVerletIntegratePositionSystem {},
            crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME,
            &[],
        );
        add_magnetics_systems_to_dispatch(&mut builder, &[]);
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut test_world);
        test_world.insert(crate::integrator::Step { n: 0 });
        test_world.insert(crate::integrator::Timestep { delta: 1.0e-6 });

        let sampler_entity = test_world.create_entity().with(NewlyCreated).build();

        dispatcher.dispatch(&test_world);
        test_world.maintain();

        let samplers = test_world.read_storage::<MagneticFieldSampler>();
        assert!(samplers.contains(sampler_entity));
    }

    // Test correct calculation of magnetic field gradient
    #[test]

    fn test_magnetic_gradient_system() {
        let mut test_world = World::new();
        register_magnetics_components(&mut test_world);
        register_magnetic_trap_components(&mut test_world);
        test_world.register::<Position>();

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(2.0, 1.0, -5.0),
            })
            .with(MagneticFieldSampler::default())
            .build();

        test_world
            .create_entity()
            .with(QuadrupoleField3D::gauss_per_cm(2.0, Vector3::z()))
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();

        test_world
            .create_entity()
            .with(QuadrupoleField3D::gauss_per_cm(1.0, Vector3::z()))
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();

        let mut quad_system = Sample3DQuadrupoleFieldSystem;
        quad_system.run_now(&test_world);

        let mut magnitude_system = CalculateMagneticFieldMagnitudeSystem;
        magnitude_system.run_now(&test_world);
        let mut gradient_system = CalculateMagneticMagnitudeGradientSystem;
        gradient_system.run_now(&test_world);

        test_world.maintain();
        let sampler_storage = test_world.read_storage::<MagneticFieldSampler>();

        let test_gradient = sampler_storage
            .get(atom1)
            .expect("entity not found")
            .gradient;

        assert_approx_eq!(test_gradient[0], 5.8554e-3, 1e-6_f64);
        assert_approx_eq!(test_gradient[1], 2.9277e-3, 1e-6_f64);
        assert_approx_eq!(test_gradient[2], -0.058554, 1e-6_f64);
    }
}
