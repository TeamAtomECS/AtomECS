extern crate specs;
use specs::{Join, ReadStorage, System, WriteStorage};

use super::cooling::{CoolingLight, CoolingLightIndex};
use super::gaussian::GaussianBeam;
use super::sampler::LaserSamplers;
use crate::atom::Force;
use crate::constant::{HBAR, PI};
use crate::initiate::AtomInfo;
use crate::magnetic::MagneticFieldSampler;
use crate::maths;

/// This sytem calculates the forces exerted by `CoolingLight` on entities.
pub struct CalculateCoolingForcesSystem;
impl<'a> System<'a> for CalculateCoolingForcesSystem {
    type SystemData = (
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, CoolingLightIndex>,
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, LaserSamplers>,
        ReadStorage<'a, AtomInfo>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (laser, laser_indices, beams, magnetic_samplers, laser_samplers, atom_info, mut forces): Self::SystemData,
    ) {
        // Outer loop over atoms
        for (atom_info, bfield, laser_sampler, mut force) in
            (&atom_info, &magnetic_samplers, &laser_samplers, &mut forces).join()
        {
            // Inner loop over cooling lasers
            for (laser, laser_index, beam) in (&laser, &laser_indices, &beams).join() {
                let s0 = laser_sampler.contents[laser_index.index].intensity
                    / atom_info.saturation_intensity;
                let detuning = laser.frequency()
                    - atom_info.frequency * 2.0 * PI
                    - laser_sampler.contents[laser_index.index].doppler_shift;
                let wavevector = maths::array_multiply(&beam.direction, laser.wavenumber());
                let costheta = maths::dot_product(&wavevector, &bfield.field)
                    / maths::modulus(&wavevector)
                    / maths::modulus(&bfield.field);
                let gamma = atom_info.gamma;
                let scatter1 = 0.25 * (laser.polarization * costheta + 1.).powf(2.) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (detuning - atom_info.mup / HBAR * bfield.magnitude).powf(2.)
                            / gamma.powf(2.));
                let scatter2 = 0.25 * (laser.polarization * costheta - 1.).powf(2.) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (detuning - atom_info.mum / HBAR * bfield.magnitude).powf(2.)
                            / gamma.powf(2.));
                let scatter3 = 0.5 * (1. - costheta.powf(2.)) * gamma
                    / 2.
                    / (1.
                        + s0
                        + 4. * (detuning - atom_info.muz / HBAR * bfield.magnitude).powf(2.)
                            / gamma.powf(2.));
                let cooling_force = maths::array_multiply(
                    &wavevector,
                    s0 * HBAR * (scatter1 + scatter2 + scatter3),
                );
                force.force = maths::array_addition(&force.force, &cooling_force);
            }
        }
    }
}
