/// Module for calculating doppler shift
extern crate specs;
use super::cooling::{CoolingLight, CoolingLightIndex};
use super::gaussian::GaussianBeam;
use super::sampler::LaserSamplers;
use crate::atom::Velocity;
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
                sampler.contents[index.index].doppler_shift = 
                vel.vel.dot(&(gaussian.direction * cooling.wavenumber()));
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    extern crate specs;
    use crate::constant::PI;
    use crate::laser::cooling::{CoolingLight, CoolingLightIndex};
    use crate::laser::sampler::{LaserSampler, LaserSamplers};
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    #[test]
    fn test_calculate_doppler_shift_system() {
        let mut test_world = World::new();
        test_world.register::<CoolingLightIndex>();
        test_world.register::<CoolingLight>();
        test_world.register::<GaussianBeam>();
        test_world.register::<Velocity>();
        test_world.register::<LaserSamplers>();

        let wavelength = 780e-9;
        test_world
            .create_entity()
            .with(CoolingLight {
                polarization: 1.0,
                wavelength: 780e-9,
            })
            .with(CoolingLightIndex { index: 0 })
            .with(GaussianBeam {
                direction: Vector3::new(1.0, 0.0, 0.0),
                intersection: Vector3::new(0.0, 0.0, 0.0),
                e_radius: 2.0,
                power: 1.0,
            })
            .build();

        let atom_velocity = 100.0;
        let sampler1 = test_world
            .create_entity()
            .with(Velocity {
                vel: Vector3::new(atom_velocity, 0.0, 0.0),
            })
            .with(LaserSamplers {
                contents: vec![LaserSampler::default()],
            })
            .build();

        let mut system = CalculateDopplerShiftSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<LaserSamplers>();

        assert_approx_eq!(
            sampler_storage
                .get(sampler1)
                .expect("entity not found")
                .contents[0]
                .doppler_shift,
            2.0 * PI / wavelength * atom_velocity,
            1e-5_f64
        );
    }
}
