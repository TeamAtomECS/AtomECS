
use crate::constant as constant;
use crate::constant::PI as PI;
use crate::integrator::{Timestep,Step};
use crate::atom::{Atom,Mass,Position,Velocity,Force,RandKick,Gravity};
use crate::initiate::{AtomInfo,NewlyCreated};
use crate::update::*;
use crate::laser::*;
use crate::magnetic::*;
use crate::initiate::atom_create::{OvenCreateAtomsSystem,Oven};
use crate::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,RunNow};
use crate::output::*;
pub fn register_resources_general(world: &mut World) {
		world.register::<Position>();
		world.register::<Velocity>();
		world.register::<Force>();
		world.register::<Mass>();
}
pub fn register_resources_atomCreation(world: &mut World) {
		world.register::<Oven>();
		world.register::<AtomInfo>();
        world.register::<Atom>();
		world.register::<NewlyCreated>();
}
pub fn register_resource_otherforce(world: &mut World) {
		world.register::<Gravity>();
		world.register::<RandKick>();
}

pub fn register_resource_output(world: &mut World) {
		world.register::<Detector>();
		world.register::<RingDetector>();
}

pub fn register_lazy(world: &mut World){
    register_resource_output(&world);
    register_resource_otherforce(&world)(&world);
    register_resources_atomCreation(&world);
    register_resources_general(&world);
    register_resources_magnetic(&world);
    register_resources_laser(&world);
}