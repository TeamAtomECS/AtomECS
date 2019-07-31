extern crate specs;
use specs::{
	DispatcherBuilder, World, Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage,
};

use crate::atom::{Force,Position,Velocity};
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
	/// The force exerted on an entity
	pub force: [f64;3],

	/// The number of photons scattered in total.
	pub scattered_photons: f64,

	/// The random kick exerted on an entity
	pub rand_force: [f64;3]
}
impl Component for CoolingForce {
	type Storage = VecStorage<Self>;
}
impl Default for CoolingForce {
	fn default() -> Self { CoolingForce { force: [0.0,0.0,0.0], scattered_photons: 0.0, rand_force: [0.0,0.0,0.0] } }
}

/// System that clears the cooling forces.
pub struct ClearCoolingForcesSystem;
impl <'a> System<'a> for ClearCoolingForcesSystem {
	type SystemData = (WriteStorage<'a,CoolingForce>);
	fn run (&mut self,mut cooling_forces:Self::SystemData){
		for cooling_forces in (&mut cooling_forces).join(){
			cooling_forces.force = [ 0.0, 0.0, 0.0];
			cooling_forces.scattered_photons = 0.0;
			cooling_forces.rand_force = [0.0,0.0,0.0];
		}
	}
}

/// System that adds the calculated cooling forces to the entity force
pub struct AddCoolingForcesSystem;
impl <'a> System<'a> for AddCoolingForcesSystem {
	type SystemData = (ReadStorage<'a,CoolingForce>, WriteStorage<'a,Force>);
	fn run (&mut self,(cooling_force, mut force):Self::SystemData){
		for (cooling_force, force) in (&cooling_force, &mut force).join(){
			force.force = maths::array_addition(&force.force, &cooling_force.force);
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
				let scattering_force = maths::array_multiply(
					&laser.wavenumber,
					s0 * HBAR * (scatter1 + scatter2 + scatter3),
				);
			cooling_force.force = maths::array_addition(&cooling_force.force, &scattering_force);
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
	);

	fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
		for (ent, _) in (&ent, &newly_created).join() {
			updater.insert(ent, CoolingForce::default());
		}
	}
}

/// Add all systems required by the laser module to the dispatch builder.
pub fn add_systems_to_dispatch(builder: DispatcherBuilder<'static,'static>, deps: &[&str]) -> DispatcherBuilder<'static,'static>  {
	builder.
	with(ClearCoolingForcesSystem,"clear_cooling_forces", deps).
	with(CalculateCoolingForcesSystem,"calculate_cooling_forces",&["clear_cooling_forces"]).
	with(AttachLaserComponentsToNewlyCreatedAtomsSystem, "", &[])
}

/// Registers all resources required by the laser module.
pub fn register_resources(world: &mut World) {
		world.register::<Laser>();
		world.register::<CoolingForce>();
}

/// Gets the intensity of a gaussian laser beam at the specified position.
fn get_gaussian_beam_intensity(laser: &Laser, pos: &Position) -> f64 {
	laser.power * maths::gaussian_dis(
		laser.std,
		maths::get_minimum_distance_line_point(&pos.pos, &laser.centre, &laser.wavenumber),
	)
}

#[cfg(test)]
pub mod tests {

	use super::*;
	extern crate specs;
	use specs::{Builder, DispatcherBuilder, World};

	/// Tests that components required for optical force calculation are added to NewlyCreated atoms
	#[test]
	fn test_laser_components_are_added_to_new_atoms() {
		let mut test_world = World::new();
		test_world.register::<NewlyCreated>();
		register_resources(&mut test_world);

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
			wavenumber: [-2.0 * constant::PI / (461e-9), 0., 0.],
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

		assert_eq!(test_world.read_storage::<CoolingForce>().contains(test_entity), true);
	}

}
