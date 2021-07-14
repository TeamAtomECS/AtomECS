//! A set of modules used for integration the motion of laser-cooled atoms.

#[macro_use]
extern crate specs_derive;

#[macro_use]
extern crate atomecs_derive;

pub mod atom;
pub mod atom_sources;
pub mod collisions;
pub mod constant;
pub mod destructor;
pub mod dipole;
pub mod ecs;
pub mod fileinput;
pub mod gravity;
pub mod helper_files;
pub mod initiate;
pub mod integration_tests;
pub mod integrator;
pub mod laser;
pub mod laser_cooling;
pub mod magnetic;
pub mod maths;
pub mod output;
pub mod ramp;
pub mod shapes;
pub mod sim_region;
