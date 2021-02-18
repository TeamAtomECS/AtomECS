//! A module for calculation forces between atoms due to rescattering.
//!
//! To enable rescattering, add a [RescatteringOption](RescatteringOption.struct.html) that specifies the configuration of rescattering forces to the simulation world.

/// A resource added to the simulation world to configure rescattering forces.
#[derive(Clone, Copy)]
pub enum RescatteringOption {
    Off,
    On(RescatteringConfiguration),
}
impl Default for RescatteringOption {
    fn default() -> Self {
        RescatteringOption::Off
    }
}

/// A particular configuration of rescattering forces.
#[derive(Clone, Copy)]
pub struct RescatteringConfiguration {
    /// The rescattering force is scaled by this amount.
    ///
    /// Scaling the force allows a simulation to model the dynamics of an otherwise intractably large number of atoms, by simulating a smaller number.
    /// The number of scattered photons per atom is scaled by this amount, and used in the repulsive force calculation.
    /// Thus, a small number of particles can model the rescattering of photons from a much brighter cloud.
    pub force_scaling: f64,

    /// Theta parameter used in the Barnes-Hut implementation, balances accuracy with speed.
    ///
    /// A value of 0 gives a direct sum. Higher values are faster but less accurate. A value of 0.5 is common.
    pub theta: f64,
}

extern crate nbody_barnes_hut;
use nbody_barnes_hut::barnes_hut_3d::OctTree;
use nbody_barnes_hut::particle_3d::Particle3D;
use nbody_barnes_hut::vector_3d::Vector3D;

extern crate nalgebra;
extern crate rayon;
extern crate specs;

use crate::atom::{AtomicTransition, Force, Position};
use crate::integrator::Timestep;
use crate::laser::photons_scattered::TotalPhotonsScattered;
use nalgebra::Vector3;
use specs::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage};

pub struct RescatteringForceSystem;
impl<'a> System<'a> for RescatteringForceSystem {
    type SystemData = (
        Option<Read<'a, RescatteringOption>>,
        ReadExpect<'a, Timestep>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, TotalPhotonsScattered>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (option, timestep, positions, transitions, scattereds, mut forces): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        match option {
            None => {}
            Some(opt) => {
                match *opt {
                    RescatteringOption::Off => {}
                    RescatteringOption::On(configuration) => {
                        // build a tree
                        let points: Vec<Particle3D> = (&positions, &scattereds)
                            .join()
                            .map(|(position, scattered)| Particle3D {
                                mass: scattered.total,
                                position: Vector3D {
                                    x: position.pos.x,
                                    y: position.pos.y,
                                    z: position.pos.z,
                                },
                            })
                            .collect();

                        let points_ref = &points.iter().collect::<Vec<&Particle3D>>()[..];
                        let scale = configuration.force_scaling;

                        let tree = OctTree::new(points_ref, configuration.theta);
                        (&mut forces, &positions)
                            .par_join()
                            .for_each(|(mut force, position)| {
                                let rescatter_force = tree.calc_forces_on_particle(
                                    Vector3D {
                                        x: position.pos.x,
                                        y: position.pos.y,
                                        z: position.pos.z,
                                    },
                                    (),
                                    |d_squared, mass, dis_vec, _| {
                                        // dis_vec is not normalized, so we have to normalize it here
                                        -mass * dis_vec / (d_squared * d_squared.sqrt())
                                    },
                                );

                                //rescattering force is scaled by the cross section
                                let cross_section =
                                    3.0 * 780e-9_f64.powi(2) / (2.0 * std::f64::consts::PI);
                                let photon_energy = crate::constant::HBAR
                                    * 2.0
                                    * std::f64::consts::PI
                                    * crate::constant::C
                                    / (780e-9_f64);

                                // The bit added in the loop above is equal to (number_photons_scattered / r^2) - rest is in prefactor.
                                let prefactor =
                                    photon_energy / (4.0 * std::f64::consts::PI) / timestep.delta
                                        * cross_section
                                        * scale
                                        / crate::constant::C;
                                force.force = force.force
                                    + Vector3::new(
                                        prefactor * rescatter_force.x,
                                        prefactor * rescatter_force.y,
                                        prefactor * rescatter_force.z,
                                    );
                            });
                    }
                }
            }
        }
    }
}
