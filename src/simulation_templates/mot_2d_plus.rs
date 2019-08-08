#[allow(unused_imports)]
use crate::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use crate::ecs;
#[allow(unused_imports)]
use crate::initiate::NewlyCreated;
use crate::laser::cooling::CoolingLight;
use crate::laser::gaussian::GaussianBeam;
use crate::magnetic::quadrupole::QuadrupoleField3D;
use crate::oven::{Oven,OvenAperture};
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
	let detuning = 6.0;
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
	world
		.create_entity()
		.with(Oven {
			temperature: 100.,
			direction: Vector3::z(),
			rate: 1e7,
			aperture:OvenAperture::Cubic{size:[1e-6,1e-6,1e-6]},
		})
		.with(AtomInfo::rubidium())
		.with(Mass { value: 87. })
		.with(Position {
			pos: Vector3::new(0.0, 0.0, -0.1),
		})
		.build();

	// Add single atom
	// world
	// 	.create_entity()
	// 	.with(Position {
	// 		pos: Vector3::new(0.001, 0.0, 0.0),
	// 	})
	// 	.with(Velocity {
	// 		vel: Vector3::new(-1.0, 0.0, 0.0),
	// 	})
	// 	.with(NewlyCreated {})
	// 	.with(Force::new())
	// 	.with(Atom {})
	// 	.with(AtomInfo::rubidium())
	// 	.with(Mass { value: 87. })
	// 	.build();
}
