// This introduces a component "CentralCreator" of the source which spawns atoms with 
// desired denstiy_distribution and velocity_distribution


extern crate nalgebra;

use super::emit::AtomNumberToEmit;
use super::precalc::{MaxwellBoltzmannSource, PrecalculatedSpeciesInformation};
use crate::constant;
use crate::constant::PI;
use crate::initiate::*;

extern crate rand;
use super::VelocityCap;
use super::WeightedProbabilityDistribution;
use rand::distributions::Distribution;
use rand::Rng;

extern crate specs;
use crate::atom::*;
use nalgebra::Vector3;

use specs::{Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System};


// Define some distributions that are necessary to custom-create the initial 
// conditions of the atoms created


#[derive(Copy, Clone)]
pub enum Position_Density_Distribution {
    Uniform_Cuboidic {size: [f64; 3]},
    Uniform_Spheric {radius: f64},
}


#[derive(Copy, Clone)]
pub enum Velocity_Density_Distribution {
    Uniform_Cuboidic {size: [f64; 3]},
    Uniform_Spheric {radius: f64},
}

#[derive(Copy, Clone)]
pub enum Spatial_Velocity_Distribution {
    Uniform {min_vel: f64, max_vel: f64},
}


/*
CentralCreator is the main structure of this script 

It is designed in analogy to the Oven but without a builder (yet, might come later)

*/
pub struct CentralCreator {
    position_density_distribution: ,
    velocity_density_distribution: ,
    spatial_velocity_distribution: ,
}

impl CentralCreator {
    pub fn new() -> Self {
        Self {

        }
    }
}