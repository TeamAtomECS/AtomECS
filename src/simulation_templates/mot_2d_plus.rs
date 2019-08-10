#[allow(unused_imports)]
use crate::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use crate::atom_sources::emit::{AtomNumberToEmit, EmitNumberPerFrame};
use crate::atom_sources::mass::{MassDistribution, MassRatio};
use crate::atom_sources::oven::{Oven, OvenAperture};
use crate::destructor::ToBeDestroyed;
use crate::ecs;
#[allow(unused_imports)]
use crate::destructor::ToBeDestroyed;
use crate::ecs;
use crate::laser::cooling::CoolingLight;
use crate::laser::gaussian::GaussianBeam;
use crate::magnetic::quadrupole::QuadrupoleField3D;
use specs::{Builder, Dispatcher, World};
extern crate nalgebra;
use nalgebra::Vector3;

/// Creates a world describing a 2D plus MOT and the dispatcher.
#[allow(dead_code)]
pub fn create() -> (World, Dispatcher<'static, 'static>) {
	let mut world = World::new();
	ecs::register_components(&mut world);
	ecs::register_resources(&mut world);

	let mut dispatcher = ecs::create_simulation_dispatcher();
	dispatcher.setup(&mut world.res);

	//component for the experiment
	mot2d_entity_create(&mut world);

	(world, dispatcher)
}

fn mot2d_entity_create(world: &mut World) {
	// Add quadrupole gradient
	let quadrupole = QuadrupoleField3D::gauss_per_cm(25.0);
	world
		.create_entity()
		.with(quadrupole)
		.with(Position::new())
		.build();

	// Add lasers
	let detuning = 150.0;
	world
		.create_entity()
		.with(GaussianBeam {
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 0.01,
			power: 1.0,
			direction: Vector3::x(),
		})
		.with(CoolingLight::for_species(
			AtomInfo::rubidium(),
			-detuning,
			1.0,
		))
		.build();
	world
		.create_entity()
		.with(GaussianBeam {
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 0.01,
			power: 1.0,
			direction: -Vector3::x(),
		})
		.with(CoolingLight::for_species(
			AtomInfo::rubidium(),
			-detuning,
			1.0,
		))
		.build();
	world
		.create_entity()
		.with(GaussianBeam {
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 0.01,
			power: 1.0,
			direction: Vector3::y(),
		})
		.with(CoolingLight::for_species(
			AtomInfo::rubidium(),
			-detuning,
			1.0,
		))
		.build();
	world
		.create_entity()
		.with(GaussianBeam {
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 0.01,
			power: 1.0,
			direction: -Vector3::y(),
		})
		.with(CoolingLight::for_species(
			AtomInfo::rubidium(),
			-detuning,
			1.0,
		))
		.build();
	world
		.create_entity()
		.with(GaussianBeam {
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 0.01,
			power: 1.0,
			direction: Vector3::z(),
		})
		.with(CoolingLight::for_species(
			AtomInfo::rubidium(),
			-detuning,
			-1.0,
		))
		.build();
	world
		.create_entity()
		.with(GaussianBeam {
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 0.01,
			power: 1.0,
			direction: -Vector3::z(),
		})
		.with(CoolingLight::for_species(
			AtomInfo::rubidium(),
			-detuning,
			-1.0,
		))
		.build();

	// Add oven
	let massrubidium = MassDistribution::new(vec![
		MassRatio {
			mass: 87.,
			ratio: 0.2783,
		},
		MassRatio {
			mass: 85.,
			ratio: 0.7217,
		},
	]);
	world
		.create_entity()
		.with(Oven {
			temperature: 100.,
			direction: Vector3::z(),
			aperture: OvenAperture::Cubic {
				size: [1e-9, 1e-9, 1e-9],
			},
		})
		.with(EmitNumberPerFrame { number: 1 })
		.with(AtomNumberToEmit { number: 0 })
		.with(AtomInfo::rubidium())
		.with(ToBeDestroyed)
		.with(massrubidium)
		.with(Position {
			pos: Vector3::new(0.0, 0.0, 0.0),
		})
		.build();
}
