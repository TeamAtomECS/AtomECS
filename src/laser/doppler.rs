/// Module for calculating doppler shift
extern crate specs;
use super::cooling::{CoolingLight, CoolingLightIndex};
use super::gaussian::GaussianBeam;
use super::sampler::LaserSamplers;
use crate::atom::Velocity;
use crate::maths;
use specs::{Join, ReadStorage, System, WriteStorage}; //todo - change for a Direction component

/// This system calculates the doppler shift for each atom in each cooling beam.
pub struct CalculateDopplerShiftSystem;
impl<'a> System<'a> for CalculateDopplerShiftSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        WriteStorage<'a, LaserSamplers>,
        ReadStorage<'a, Velocity>,
    );
    fn run(&mut self, (cooling, indices, gaussian, mut samplers, velocities): Self::SystemData) {
        for (cooling, index, gaussian) in (&cooling, &indices, &gaussian).join() {
            for (sampler, vel) in (&mut samplers, &velocities).join() {
                sampler.contents[index.index].doppler_shift = maths::dot_product(
                    &maths::array_multiply(&gaussian.direction, cooling.wavenumber()),
                    &vel.vel);
            }
        }
    }
}
