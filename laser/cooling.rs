extern crate specs;
use specs::{
	DispatcherBuilder, World, Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage, ReadExpect, HashMapStorage, Entity
};

use crate::atom::{Force,Position,Velocity};
use crate::constant;
use crate::constant::HBAR;
use crate::initiate::{AtomInfo, NewlyCreated};
use crate::magnetic::*;
use crate::maths;
use crate::integrator::Timestep;
use rand::Rng;

/// A component representing light used for laser cooling.
pub struct CoolingLight {

	/// Polarisation of the laser light, 1. for +, -1. for -,
	pub polarization: f64,

	/// wavelength of the laser light, in SI units of m.
	pub wavelength: f64

}
impl CoolingLight {

	/// Frequency of the cooling light in units of Hz
	pub fn frequency(&self) -> f64 { constant::C/self.wavelength }
	
	/// Wavenumber of the cooling light, in units of inverse metres.
	pub fn wavenumber(&self) -> f64 { 2 * constant::PI / self.wavelength }

}
impl Component for CoolingLight {
	type Storage = HashMapStorage<Self>;
}

/// This component holds a vector list of all interactions of the entity with each laser beam in the simulation. 
pub struct CoolingLaserInteractions {
	pub list: Vector<CoolingForce>
}
impl Component for CoolingLaserInteractions {
	type Storage = VecStorage<Self>;
}