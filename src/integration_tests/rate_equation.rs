//! Integration tests for the rate equation approach
//!
//! This module tests the rate equation implementation in atomecs by comparison to the exact analytic results for a single beam.

#[cfg(test)]
pub mod tests {
    use crate::atom::{Atom, Force, Mass, Position, Velocity};
    use crate::initiate::NewlyCreated;
    use crate::integrator::Timestep;
    use crate::laser::gaussian::GaussianBeam;
    use crate::laser::index::LaserIndex;
    use crate::laser::LaserPlugin;
    use crate::laser_cooling::photons_scattered::TotalPhotonsScattered;
    use crate::laser_cooling::transition::AtomicTransition;
    use crate::laser_cooling::{CoolingLight, LaserCoolingPlugin};
    use crate::simulation::SimulationBuilder;
    use crate::species::Rubidium87_780D2;
    extern crate nalgebra;
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;
    use specs::prelude::*;

    #[test]
    fn single_beam_scattering_rates_v_detuning() {
        test_single_beam_scattering_rate(1.0, -2.0);
        test_single_beam_scattering_rate(1.0, -1.0);
        test_single_beam_scattering_rate(1.0, 0.0);
        test_single_beam_scattering_rate(1.0, 1.0);
        test_single_beam_scattering_rate(1.0, 2.0);
    }

    #[test]
    fn single_beam_scattering_rates_v_intensity() {
        test_single_beam_scattering_rate(1.0, 0.0);
        test_single_beam_scattering_rate(2.0, 0.0);
        test_single_beam_scattering_rate(3.0, 0.0);
        test_single_beam_scattering_rate(4.0, 0.0);
        test_single_beam_scattering_rate(5.0, 0.0);
    }

    /// Calculates the scattering rate from a single beam at given intensity and detuning, and compares that to analytic theory.
    fn test_single_beam_scattering_rate(i_over_i_sat: f64, delta_over_gamma: f64) {
        const BEAM_NUMBER: usize = 1;
        let transition = Rubidium87_780D2;
        let i_sat = Rubidium87_780D2::saturation_intensity();
        let intensity = i_sat * i_over_i_sat;
        let delta = delta_over_gamma * Rubidium87_780D2::gamma();
        let detuning_megahz = delta / (2.0 * std::f64::consts::PI * 1.0e6);

        // Create simulation dispatcher
        let mut sim_builder = SimulationBuilder::default();
        sim_builder.add_plugin(LaserPlugin::<{ BEAM_NUMBER }>);
        sim_builder.add_plugin(LaserCoolingPlugin::<Rubidium87_780D2, { BEAM_NUMBER }>::default());
        let mut sim = sim_builder.build();

        // add laser to test world.
        sim.world
            .create_entity()
            .with(CoolingLight::for_transition::<Rubidium87_780D2>(
                detuning_megahz,
                1,
            ))
            .with(LaserIndex::default())
            .with(GaussianBeam::from_peak_intensity_with_rayleigh_range(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                intensity,
                0.01,
                780.0e-9,
            ))
            .build();

        // Configure timestep to be one us so that calculated rates are MHz.
        let dt = 1.0e-6;
        sim.world.insert(Timestep { delta: dt });

        // add an atom to the world. We don't add force nor mass, because we don't need them.
        let atom = sim
            .world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(transition)
            .with(Atom)
            .with(NewlyCreated)
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .build();

        sim.world
            .create_entity()
            .with(crate::magnetic::uniform::UniformMagneticField::gauss(
                Vector3::new(0.1, 0.0, 0.0),
            ))
            .build();

        // The first step is to add required components to new atoms.
        sim.step();

        // Reset position and velocity to zero.
        assert!(sim
            .world
            .write_storage::<Position>()
            .insert(
                atom,
                Position {
                    pos: Vector3::new(0.0, 0.0, 0.0),
                },
            )
            .is_ok());
        assert!(sim
            .world
            .write_storage::<Velocity>()
            .insert(
                atom,
                Velocity {
                    vel: Vector3::new(0.0, 0.0, 0.0),
                },
            )
            .is_ok());

        // Second step to calculate values over completed atoms.
        sim.step();

        let expected_scattered =
            analytic_scattering_rate(intensity, i_sat, delta, Rubidium87_780D2::gamma());
        let total_scattered = sim
            .world
            .read_storage::<TotalPhotonsScattered<Rubidium87_780D2>>()
            .get(atom)
            .expect("Could not find atom in storage.")
            .total
            / dt;
        assert_approx_eq!(
            total_scattered,
            expected_scattered,
            expected_scattered.abs() * 0.05
        );

        // Compare the magnitude of the calculated force.
        let k = 2.0 * std::f64::consts::PI * Rubidium87_780D2::frequency() / crate::constant::C;
        let photon_momentum = crate::constant::HBAR * k;
        let analytic_force = (expected_scattered * dt) * photon_momentum / dt;
        let measured_force = sim
            .world
            .read_storage::<Force>()
            .get(atom)
            .expect("Atom does not have force component.")
            .force;
        assert_approx_eq!(
            measured_force.norm(),
            analytic_force,
            analytic_force.abs() * 0.04
        );
    }

    /// Analytic scattering rate for a two-level system. Returns photon-scattering rate in units of Hz.
    fn analytic_scattering_rate(intensity: f64, i_sat: f64, delta: f64, gamma: f64) -> f64 {
        let i_over_i_sat = intensity / i_sat;
        (gamma / 2.0) * i_over_i_sat / (1.0 + i_over_i_sat + 4.0 * (delta / gamma).powi(2))
    }
}
