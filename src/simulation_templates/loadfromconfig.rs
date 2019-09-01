
#[allow(unused_imports)]
use crate::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use crate::atom_sources::emit::{AtomNumberToEmit, EmitFixedRate, EmitNumberPerFrame};
use crate::atom_sources::oven::{Oven, OvenAperture};
use crate::constant;

use crate::destructor::ToBeDestroyed;
use crate::detector::{ClearerCSV, Detector};
use crate::ecs;
use crate::laser::cooling::CoolingLight;
use crate::laser::gaussian::GaussianBeam;
use crate::magnetic::quadrupole::QuadrupoleField3D;
use crate::magnetic::uniform::UniformMagneticField;

use crate::integrator::Timestep;

use crate::fileinput::load_file;
use specs::{Builder, Dispatcher, World};
extern crate nalgebra;

pub fn create_from_config(filename: &str) -> (World, Dispatcher<'static, 'static>) {
	let mut world = World::new();
	ecs::register_components(&mut world);
	ecs::register_resources(&mut world);
	let mut dispatcher = ecs::create_simulation_dispatcher();
	dispatcher.setup(&mut world.res);
	create_simulation_entity(&filename, &mut world);

	(world, dispatcher)
}
pub fn create_simulation_entity(filename: &str, world: &mut World) {
	let config = load_file(&filename);
	for laser in config.lasers.iter() {
		world
			.create_entity()
			.with(GaussianBeam {
				intersection: laser.intersection,
				e_radius: laser.e_radius,
				power: laser.power,
				direction: laser.direction,
			})
			.with(CoolingLight {
				polarization: laser.polarization,
				wavelength: constant::C / laser.frequency,
			})
			.build();
	}
	let mut mass = config.mass.clone();
	mass.normalise();
	println!("{:?}", mass.normalised);
	for oven in config.ovens.iter() {
		world
			.create_entity()
			.with(Oven {
				temperature: oven.temperature,
				direction: oven.direction.clone(),
				aperture: OvenAperture::Circular {
					radius: oven.radius_aperture,
					thickness: oven.thickness,
				},
			})
			.with(AtomNumberToEmit { number: 0 })
			.with(EmitFixedRate { rate: oven.rate })
			.with(config.atominfo.clone())
			.with(mass.clone())
			.with(Position { pos: oven.position })
			.build();
		if oven.instant_emission != 0 {
			world
				.create_entity()
				.with(Oven {
					temperature: oven.temperature,
					direction: oven.direction.clone(),
					aperture: OvenAperture::Circular {
						radius: oven.radius_aperture,
						thickness: oven.thickness,
					},
				})
				.with(AtomNumberToEmit { number: 0 })
				.with(EmitNumberPerFrame {
					number: oven.instant_emission as i32,
				})
				.with(ToBeDestroyed)
				.with(config.atominfo.clone())
				.with(mass.clone())
				.with(Position { pos: oven.position })
				.build();
		}
	}
	let quadrupole = QuadrupoleField3D::gauss_per_cm(config.magnetic.gradient);
	world
		.create_entity()
		.with(quadrupole)
		.with(Position {
			pos: config.magnetic.centre,
		})
		.build();

	world
		.create_entity()
		.with(UniformMagneticField {
			field: config.magnetic.uniform,
		})
		.build();
	world
		.create_entity()
		.with(Detector {
			filename: "detector.csv",
			direction: config.detector.direction.clone(),
			radius: config.detector.radius,
			thickness: config.detector.thickness,
			trigger_time: config.detector.trigger_time,
		})
		.with(Position {
			pos: config.detector.position.clone(),
		})
		.build();
	world
		.create_entity()
		.with(ClearerCSV {
			filename: "detector.csv",
		})
		.build();

	world.add_resource(Timestep {
		delta: config.timestep,
	});
}