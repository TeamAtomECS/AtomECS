//! Magnetic fields and zeeman shift

extern crate nalgebra;
extern crate specs;
use crate::initiate::NewlyCreated;
use crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME;
use nalgebra::Vector3;
use specs::{
	Component, DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadStorage, System,
	VecStorage, World, WriteStorage,
};

pub mod grid;
pub mod quadrupole;
pub mod uniform;
pub mod zeeman;
use std::fmt;

/// A component that stores the magnetic field at an entity's location.
#[derive(Copy, Clone)]
pub struct MagneticFieldSampler {
	/// Vector representing the magnetic field components along x,y,z in units of Tesla.
	pub field: Vector3<f64>,

	/// Magnitude of the magnetic field in units of Tesla
	pub magnitude: f64,
}
impl MagneticFieldSampler {
	pub fn tesla(b_field: Vector3<f64>) -> Self {
		MagneticFieldSampler {
			field: b_field,
			magnitude: b_field.norm(),
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
		}
	}
}

/// System that clears the magnetic field samplers each frame.
pub struct ClearMagneticFieldSamplerSystem;

impl<'a> System<'a> for ClearMagneticFieldSamplerSystem {
	type SystemData = WriteStorage<'a, MagneticFieldSampler>;
	fn run(&mut self, mut sampler: Self::SystemData) {
		use rayon::prelude::*;
		use specs::ParJoin;

		(&mut sampler).par_join().for_each(|mut sampler| {
			sampler.magnitude = 0.;
			sampler.field = Vector3::new(0.0, 0.0, 0.0)
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
		use specs::ParJoin;

		(&mut sampler).par_join().for_each(|mut sampler| {
			sampler.magnitude = sampler.field.norm();
			if sampler.magnitude.is_nan() {
				sampler.magnitude = 0.0;
			}
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
pub fn add_systems_to_dispatch(builder: &mut DispatcherBuilder<'static, 'static>, deps: &[&str]) {
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
		grid::SampleMagneticGridSystem,
		"magnetics_grid",
		&["magnetics_uniform", INTEGRATE_POSITION_SYSTEM_NAME],
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
	builder.add(
		zeeman::AttachZeemanShiftSamplersToNewlyCreatedAtomsSystem,
		"attach_zeeman_shift_samplers",
		&[],
	);
	builder.add(
		zeeman::CalculateZeemanShiftSystem,
		"zeeman_shift",
		&["magnetics_magnitude"],
	);
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<uniform::UniformMagneticField>();
	world.register::<quadrupole::QuadrupoleField3D>();
	world.register::<quadrupole::QuadrupoleField2D>();
	world.register::<MagneticFieldSampler>();
	world.register::<grid::PrecalculatedMagneticFieldGrid>();
}

#[cfg(test)]
pub mod tests {

	use super::*;
	extern crate specs;
	use crate::atom::Position;
	use specs::{Builder, DispatcherBuilder, World};

	/// Tests the correct implementation of the magnetics systems and dispatcher.
	/// This is done by setting up a test world and ensuring that the magnetic systems perform the correct operations on test entities.
	#[test]
	fn test_magnetics_systems() {
		let mut test_world = World::new();
		register_components(&mut test_world);
		test_world.register::<NewlyCreated>();
		test_world.register::<zeeman::ZeemanShiftSampler>();
		let mut builder = DispatcherBuilder::new();
		builder.add(
			crate::integrator::VelocityVerletIntegratePositionSystem {},
			crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME,
			&[],
		);
		add_systems_to_dispatch(&mut builder, &[]);
		let mut dispatcher = builder.build();
		dispatcher.setup(&mut test_world.res);
		test_world.add_resource(crate::integrator::Step { n: 0 });
		test_world.add_resource(crate::integrator::Timestep { delta: 1.0e-6 });

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

		dispatcher.dispatch(&mut test_world.res);

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
		register_components(&mut test_world);
		test_world.register::<NewlyCreated>();
		test_world.register::<zeeman::ZeemanShiftSampler>();
		let mut builder = DispatcherBuilder::new();
		builder.add(
			crate::integrator::VelocityVerletIntegratePositionSystem {},
			crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME,
			&[],
		);
		add_systems_to_dispatch(&mut builder, &[]);
		let mut dispatcher = builder.build();
		dispatcher.setup(&mut test_world.res);
		test_world.add_resource(crate::integrator::Step { n: 0 });
		test_world.add_resource(crate::integrator::Timestep { delta: 1.0e-6 });

		let sampler_entity = test_world.create_entity().with(NewlyCreated).build();

		dispatcher.dispatch(&mut test_world.res);
		test_world.maintain();

		let samplers = test_world.read_storage::<MagneticFieldSampler>();
		assert_eq!(samplers.contains(sampler_entity), true);
	}
}
