extern crate specs;
use specs::{
	DispatcherBuilder, World, Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage,
};

use crate::atom::*;
use crate::constant;
use crate::constant::HBAR;
use crate::initiate::{AtomInfo, NewlyCreated};
use crate::magnetic::*;
use crate::maths;

/// Represents a laser beam that is used to provide cooling forces to atoms in the simulation.
pub struct Laser {
	
	/// A point that lies on the laser beam
	pub centre: [f64; 3],

	/// wavevector of the laser light in SI unit. 
	/// This quantity is a vector, which points along the direction of the laser beam.
	pub wavenumber: [f64; 3],
	
	/// polarisation of the laser light, 1. for +, -1. for -,
	pub polarization: f64,
	
	/// power of the laser in W
	pub power: f64,
	
	/// stand deviation of the laser light gaussian distribution, SI units of metres
	pub std: f64,
	
	/// frequency of the laser light
	pub frequency: f64,
	
	/// index of the laser light, it is used to record the interaction between any laser and any atom
	pub index: u64,
}

impl Component for Laser {
	type Storage = VecStorage<Self>;
}

/// Cooling force exerted on an entity by the cooling laser beams
pub struct CoolingForce {
	pub force: [f64;3]
}

impl Component for CoolingForce {
	type Storage = VecStorage<Self>;
}


/// System that clears the cooling forces.
pub struct ClearCoolingForcesSystem;
impl <'a> System<'a> for ClearCoolingForcesSystem {
	type SystemData = (WriteStorage<'a,CoolingForce>);
	fn run (&mut self,mut cooling_forces:Self::SystemData){
		for cooling_forces in (&mut cooling_forces).join(){
			cooling_forces.force = [ 0.0, 0.0, 0.0];
		}
	}
}

/// This system calculates cooling forces exerted by the cooling lasers on atoms.
pub struct CalculateCoolingForcesSystem;
impl<'a> System<'a> for CalculateCoolingForcesSystem {
	
	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, Laser>,
		ReadStorage<'a, Velocity>,
		ReadStorage<'a, MagneticFieldSampler>,
		WriteStorage<'a, CoolingForce>,
		ReadStorage<'a, AtomInfo>,
	);

	fn run(&mut self, (pos, laser, vel, mag, mut cooling_force, atom): Self::SystemData) {

		// Outer loop over laser beams
		for laser in (&laser).join()
		{
		// Inner loop over atoms
		for (vel, pos, mag, mut cooling_force, atom) in
			(&vel, &pos, &mag, &mut cooling_force, &atom).join()
		{
			let br = mag.magnitude;

			let laser_intensity = get_gaussian_beam_intensity(&laser, &pos);
			let s0 = laser_intensity / atom.saturation_intensity;
			let laser_omega = maths::modulus(&laser.wavenumber) * constant::C;


				let costheta = maths::dot_product(&laser.wavenumber, &mag.field)
					/ maths::modulus(&laser.wavenumber)
					/ maths::modulus(&mag.field);
				let detuning = laser_omega
					- atom.frequency * 2.0 * constant::PI
					- maths::dot_product(&laser.wavenumber, &vel.vel);

				let scatter1 =
					0.25 * (laser.polarization * costheta + 1.).powf(2.) * atom.gamma
						/ 2. / (1. + s0 + 4. * (detuning - atom.mup / HBAR * br).powf(2.) / atom.gamma.powf(2.));
				let scatter2 =
					0.25 * (laser.polarization * costheta - 1.).powf(2.) * atom.gamma
						/ 2. / (1. + s0 + 4. * (detuning - atom.mum / HBAR * br).powf(2.) / atom.gamma.powf(2.));
				let scatter3 =
					0.5 * (1. - costheta.powf(2.)) * atom.gamma
						/ 2. / (1. + s0 + 4. * (detuning - atom.muz / HBAR * br).powf(2.) / atom.gamma.powf(2.));
				let force_new = maths::array_multiply(
					&laser.wavenumber,
					s0 * HBAR * (scatter1 + scatter2 + scatter3),
				);
			}
		}
	}
}

fn get_perpen_distance(pos: &[f64; 3], centre: &[f64; 3], dir: &[f64; 3]) -> f64 {
	let rela_cood = maths::array_addition(&pos, &maths::array_multiply(&centre, -1.));
	let distance = maths::modulus(&maths::cross_product(&dir, &rela_cood)) / maths::modulus(&dir);
	distance
}

/// Gets the intensity of a gaussian laser beam at the specified position.
fn get_gaussian_beam_intensity(laser: &Laser, pos: &Position) -> f64 {
	laser.power * maths::gaussian_dis(
		laser.std,
		get_perpen_distance(&pos.pos, &laser.centre, &laser.wavenumber),
	)
}


pub struct InteractionLaser {
	/// which laser is involved
	pub index: u64,
	/// intensity of the this laser light at this position
	pub intensity: f64,
	pub polarization: f64,
	pub wavenumber: [f64; 3],
	/// the detuning between the laser light and the atom
	pub detuning_doppler: f64,
	pub force: [f64; 3],
}

impl InteractionLaser {
	pub fn clone(&self) -> InteractionLaser {
		InteractionLaser {
			index: self.index,
			intensity: self.intensity,
			polarization: self.polarization,
			wavenumber: self.wavenumber.clone(),
			detuning_doppler: self.detuning_doppler,
			force: self.force.clone(),
		}
	}
}

pub struct InteractionLaserALL {
	// just a collection of laser interactions
	pub content: Vec<InteractionLaser>,
}

impl Component for InteractionLaserALL {
	type Storage = VecStorage<Self>;
}

impl InteractionLaserALL {
	pub fn clone(&self) -> InteractionLaserALL {
		let mut new = Vec::new();
		for i in self.content.iter() {
			new.push(i.clone());
		}
		InteractionLaserALL { content: new }
	}
}

pub struct UpdateInteractionLaserSystem;
impl<'a> System<'a> for UpdateInteractionLaserSystem {
	// this system will update the information regarding interaction between the lasers and the atoms
	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, Velocity>,
		ReadStorage<'a, MagneticFieldSampler>,
		WriteStorage<'a, InteractionLaserALL>,
		ReadStorage<'a, AtomInfo>,
	);

	fn run(&mut self, (_pos, vel, mag, mut interall, atom): Self::SystemData) {
		for (vel, _pos, mag, mut interall, atom) in
			(&vel, &_pos, &mag, &mut interall, &atom).join()
		{
			//println!("laser interaction updated");
			let mag_field = mag.field;
			let br = mag.magnitude;
			for inter in &mut interall.content {
				let _mup = atom.mup;
				let _mum = atom.mum;
				let _muz = atom.muz;
				let s0 = inter.intensity / constant::SATINTEN;
				let omega = maths::modulus(&inter.wavenumber) * constant::C;
				let wave_vector = inter.wavenumber;
				let p = inter.polarization;
				let gamma = atom.gamma;
				let atom_frequency = atom.frequency;
				let costheta = maths::dot_product(&wave_vector, &mag_field)
					/ maths::modulus(&wave_vector)
					/ maths::modulus(&mag_field);
				let detuning = omega
					- atom_frequency * 2.0 * constant::PI
					- maths::dot_product(&wave_vector, &vel.vel);

				let scatter1 =
					0.25 * (p * costheta + 1.).powf(2.) * gamma
						/ 2. / (1. + s0 + 4. * (detuning - _mup / HBAR * br).powf(2.) / gamma.powf(2.));
				let scatter2 =
					0.25 * (p * costheta - 1.).powf(2.) * gamma
						/ 2. / (1. + s0 + 4. * (detuning - _mum / HBAR * br).powf(2.) / gamma.powf(2.));
				let scatter3 =
					0.5 * (1. - costheta.powf(2.)) * gamma
						/ 2. / (1. + s0 + 4. * (detuning - _muz / HBAR * br).powf(2.) / gamma.powf(2.));
				let force_new = maths::array_multiply(
					&wave_vector,
					s0 * HBAR * (scatter1 + scatter2 + scatter3),
				);

				inter.force = force_new;
				inter.detuning_doppler = detuning;
			}
		}
	}
}

pub struct UpdateLaserSystem;

impl<'a> System<'a> for UpdateLaserSystem {
	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, Laser>,
		WriteStorage<'a, InteractionLaserALL>,
	);

	fn run(&mut self, (pos, _laser, mut interall): Self::SystemData) {
		//update the sampler for laser, namely intensity, wavenumber? , polarization
		for (mut interall, pos) in (&mut interall, &pos).join() {
			//println!("laser updated");
			for inter in &mut interall.content {
				for _laser in (&_laser).join() {
					if _laser.index == inter.index {
						let laser_inten = _laser.power
							* maths::gaussian_dis(
								_laser.std,
								get_perpen_distance(&pos.pos, &_laser.centre, &_laser.wavenumber),
							);
						inter.intensity = laser_inten;
						inter.wavenumber = _laser.wavenumber;
						inter.polarization = _laser.polarization;
					}
				}
			}
		}
	}
}

/// Attachs components used for optical force calculation to newly created atoms.
///
/// This system attaches the `RandKick` and `InteractionLaserALL` components to `NewlyCreated` entities.
/// Both components are required by other laser `System`s to perform calculations of optical scattering forces.
pub struct AttachLaserComponentsToNewlyCreatedAtomsSystem;

impl<'a> System<'a> for AttachLaserComponentsToNewlyCreatedAtomsSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, NewlyCreated>,
		Read<'a, LazyUpdate>,
		ReadStorage<'a, Laser>,
	);

	fn run(&mut self, (ent, newly_created, updater, laser): Self::SystemData) {
		let mut content = Vec::new();
		for laser in (&laser).join() {
			content.push(InteractionLaser {
				index: laser.index,
				intensity: 0.,
				polarization: 0.,
				wavenumber: [0., 0., 0.],
				detuning_doppler: 0.,
				force: [0., 0., 0.],
			})
		}

		let laser_interaction = InteractionLaserALL { content };
		for (ent, _) in (&ent, &newly_created).join() {
			updater.insert(
				ent,
				RandKick {
					force: [0., 0., 0.],
				},
			);
			updater.insert(ent, laser_interaction.clone());
		}
	}
}

/// add all Systems that are necessary to run the laser part of the simulation
pub fn add_systems_to_dispatch_laser(builder: DispatcherBuilder<'static,'static>, deps: &[&str]) -> DispatcherBuilder<'static,'static> {
	builder.
	with(UpdateLaserSystem,"updatelaser", deps).
	with(UpdateInteractionLaserSystem,"updatelaserinter",&["updatelaser"]).
	with(AttachLaserComponentsToNewlyCreatedAtomsSystem, "add_laser", &[])
}

pub fn register_resources_laser(world: &mut World) {
		world.register::<InteractionLaserALL>();
		world.register::<Laser>();
}

#[cfg(test)]
pub mod tests {

	use super::*;
	extern crate specs;
	use specs::{Builder, DispatcherBuilder, World};

	#[test]
	fn test_gaussian_beam() {
		let pos = [1., 1., 1.];
		let centre = [0., 1., 1.];
		let dir = [1., 2., 2.];
		let distance = get_perpen_distance(&pos, &centre, &dir);
		assert_eq!(distance > 0.942, distance < 0.943);
	}

	/// Tests that components required for optical force calculation are added to NewlyCreated atoms
	#[test]
	fn test_laser_components_are_added_to_new_atoms() {
		let mut test_world = World::new();
		test_world.register::<NewlyCreated>();
		test_world.register::<RandKick>();
		test_world.register::<InteractionLaserALL>();
		test_world.register::<Laser>();

		let mut dispatcher = DispatcherBuilder::new()
			.with(
				AttachLaserComponentsToNewlyCreatedAtomsSystem,
				"attach_comps",
				&[],
			)
			.build();
		dispatcher.setup(&mut test_world.res);

		let laser = Laser {
			centre: [0., 0., 0.],
			wavenumber: [-2.0 * PI / (461e-9), 0., 0.],
			polarization: -1.,
			power: 10.,
			std: 0.1,
			frequency: constant::C / 461e-9,
			index: 6,
		};
		test_world.create_entity().with(laser).build();

		let test_entity = test_world.create_entity().with(NewlyCreated).build();

		dispatcher.dispatch(&mut test_world.res);
		test_world.maintain();

		assert_eq!(test_world.read_storage::<RandKick>().contains(test_entity), true);
		assert_eq!(test_world.read_storage::<InteractionLaserALL>().contains(test_entity), true);
	}

	#[test]
	fn test_laser_interaction() {
		use specs::{Builder, RunNow, World};
		let mut test_world = World::new();
		test_world.register::<InteractionLaserALL>();
		test_world.register::<Force>();
		test_world.register::<Laser>();
		test_world.register::<MagneticFieldSampler>();
		test_world.register::<Position>();
		test_world.register::<Velocity>();
		test_world.register::<AtomInfo>();
		let rb_atom = AtomInfo {
			mass: 87,
			mup: constant::MUP,
			mum: constant::MUM,
			muz: constant::MUZ,
			frequency: constant::ATOMFREQUENCY,
			gamma: constant::TRANSWIDTH,
		};
		let mut content = Vec::new();
		content.push(InteractionLaser {
			wavenumber: [1., 1., 2.],
			index: 1,
			intensity: 1.,
			polarization: 1.,
			detuning_doppler: 1.,
			force: [1., 0., 0.],
		});
		content.push(InteractionLaser {
			wavenumber: [1., 1., 2.],
			index: 2,
			intensity: 1.,
			polarization: 1.,
			detuning_doppler: 1.,
			force: [2., 0., 0.],
		});
		let test_interaction = InteractionLaserALL { content };
		let sample_entity = test_world
			.create_entity()
			.with(test_interaction)
			.with(MagneticFieldSampler {
				magnitude: 5.,
				field: [3., 4., 0.],
			})
			.with(rb_atom)
			.with(Position { pos: [0., 0., 0.] })
			.with(Velocity { vel: [0., 0., 0.] })
			.build();

		let _laser_1 = Laser {
			centre: [0., 0., 0.],
			wavenumber: [0.0, 0.0, 2.0 * PI / (461e-9)],
			polarization: 1.,
			power: 10.,
			std: 0.1,
			frequency: constant::C / 461e-9,
			index: 1,
		};
		let _laser_2 = Laser {
			centre: [0., 0., 0.],
			wavenumber: [0.0, 0.0, -2.0 * PI / (461e-9)],
			polarization: 1.,
			power: 10.,
			std: 0.1,
			frequency: constant::C / 461e-9,

			index: 2,
		};
		test_world.create_entity().with(_laser_1).build();
		test_world.create_entity().with(_laser_2).build();

		let mut update_test = UpdateLaserSystem;
		let mut update_test_two = UpdateInteractionLaserSystem;
		update_test.run_now(&test_world.res);
		update_test_two.run_now(&test_world.res);

		let samplers = test_world.read_storage::<InteractionLaserALL>();
		let sampler = samplers.get(sample_entity);
		assert_eq!(
			(sampler.expect("entity not found").content[0].force[2] * 1e22) as u64,
			46
		);
	}

}
