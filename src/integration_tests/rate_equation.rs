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
    use crate::laser_cooling::photons_scattered::{
        ScatteringFluctuationsOption, TotalPhotonsScattered,
    };
    use crate::laser_cooling::transition::AtomicTransition;
    use crate::laser_cooling::{CoolingLight, LaserCoolingPlugin};
    use crate::simulation;
    use crate::species::Rubidium87_780D2;
    extern crate nalgebra;
    use assert_approx_eq::assert_approx_eq;
    use bevy::prelude::*;
    use nalgebra::Vector3;

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
        let mut simulation = simulation::SimulationBuilder::default().build();
        simulation.add_plugins(LaserPlugin::<{ BEAM_NUMBER }>);
        simulation.add_plugins(LaserCoolingPlugin::<Rubidium87_780D2, { BEAM_NUMBER }>::default());

        // Disable scattering
        simulation.insert_resource(ScatteringFluctuationsOption::Off);

        // add laser to test world.
        simulation
            .world_mut()
            .spawn(CoolingLight::for_transition::<Rubidium87_780D2>(
                detuning_megahz,
                1,
            ))
            .insert(LaserIndex::default())
            .insert(GaussianBeam::from_peak_intensity_with_rayleigh_range(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                intensity,
                0.01,
                780.0e-9,
            ));

        // Configure timestep to be one us so that calculated rates are MHz.
        let dt = 1.0e-6;
        simulation.world_mut().insert_resource(Timestep { delta: dt });

        // add an atom to the world. We don't add force nor mass, because we don't need them.
        let atom = simulation
            .world_mut()
            .spawn(Position {
                pos: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.0),
            })
            .insert(transition)
            .insert(Atom)
            .insert(NewlyCreated)
            .insert(Force::default())
            .insert(Mass { value: 87.0 })
            .id();

        simulation
            .world_mut()
            .spawn(crate::magnetic::uniform::UniformMagneticField::gauss(
                Vector3::new(0.1, 0.0, 0.0),
            ));

        // The first step is to add required components to new atoms.
        simulation.update();

        // Reset position and velocity to zero.
        simulation
            .world_mut()
            .get_mut::<Position>(atom)
            .expect("Atom not found")
            .as_mut()
            .pos = Vector3::default();
        simulation
            .world_mut()
            .get_mut::<Velocity>(atom)
            .expect("Atom not found")
            .as_mut()
            .vel = Vector3::default();

        // Second step to calculate values over completed atoms.
        simulation.update();

        let expected_scattered =
            analytic_scattering_rate(intensity, i_sat, delta, Rubidium87_780D2::gamma());
        let total_scattered = simulation
            .world()
            .get::<TotalPhotonsScattered<Rubidium87_780D2>>(atom)
            .expect("Could not find atom in storage.")
            .total
            / dt;
        assert_approx_eq!(
            total_scattered,
            expected_scattered,
            expected_scattered.abs() * 0.01
        );

        // Compare the magnitude of the calculated force.
        let k = 2.0 * std::f64::consts::PI * Rubidium87_780D2::frequency() / crate::constant::C;
        let photon_momentum = crate::constant::HBAR * k;
        let analytic_force = (expected_scattered * dt) * photon_momentum / dt;
        let measured_force = simulation
            .world()
            .get::<Force>(atom)
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
