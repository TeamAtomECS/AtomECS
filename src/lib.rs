//! # AtomECS
//!
//! AtomECS is a high-performance code for simulating the motion of ultracold atoms.
//!
//! See the repository [readme](https://github.com/TeamAtomECS/AtomEC) for more information.

#[macro_use]
extern crate atomecs_derive;

pub mod atom;
pub mod atom_sources;
pub mod collisions;
pub mod constant;
pub mod destructor;
pub mod dipole;
pub mod bevy_bridge;
pub mod gravity;
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
pub mod simulation;
pub mod species;
