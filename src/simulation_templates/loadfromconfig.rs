
#[allow(unused_imports)]
use crate::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use crate::atom_sources::emit::{AtomNumberToEmit, EmitFixedRate};
use crate::atom_sources::oven::{Oven, OvenAperture};
use crate::constant;
use crate::ecs;
use crate::destructor::ToBeDestroyed;
use crate::fileinput::load_file;
use crate::initiate::NewlyCreated;
use crate::laser::cooling::CoolingLight;
use crate::laser::gaussian::GaussianBeam;
use crate::magnetic::quadrupole::QuadrupoleField3D;
use crate::magnetic::uniform::UniformMagneticField;

use specs::{Builder, Dispatcher, World};
extern crate nalgebra;

pub fn create_from_config() -> (World, Dispatcher<'static, 'static>) {
	let mut world = World::new();
	ecs::register_components(&mut world);
	ecs::register_resources(&mut world);
	let mut dispatcher = ecs::create_simulation_dispatcher();
	dispatcher.setup(&mut world.res);
	create_simulation_entity("example.yaml", &mut world);

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
	for oven in config.ovens.iter() {
		world
			.create_entity()
			.with(Oven {
				temperature: oven.temperature,
				direction: oven.direction,
				aperture: OvenAperture::Circular {
					radius: oven.radius_aperture,
					thickness: oven.thickness,
				},
			})
			.with(AtomNumberToEmit { number: 0 })
			.with(EmitFixedRate { rate: oven.rate })
			.with(config.atominfo.clone())
			.with(config.mass.clone())
			.with(Position { pos: oven.position })
			.build();
		if oven.instant_emission !=0 {
			world
				.create_entity()
				.with(Oven {
					temperature: oven.temperature,
					direction: oven.direction,
					aperture: OvenAperture::Circular {
						radius: oven.radius_aperture,
						thickness: oven.thickness,
					},
				})
				.with(AtomNumberToEmit { number: oven.instant_emission as i32 })
				.with(EmitFixedRate { rate: 0. })
				.with(ToBeDestroyed)
				.with(config.atominfo.clone())
				.with(config.mass.clone())
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

}