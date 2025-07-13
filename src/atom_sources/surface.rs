//! Surface sources

extern crate nalgebra;
use std::marker::PhantomData;

use nalgebra::Vector3;

use super::emit::AtomNumberToEmit;
use super::VelocityCap;
use super::species::AtomCreator;
use rand;
use rand::Rng;

use super::precalc::{MaxwellBoltzmannSource, PrecalculatedSpeciesInformation};
use crate::atom::*;
use crate::initiate::NewlyCreated;
use crate::shapes::{Cylinder, Surface};

extern crate bevy;
use bevy::prelude::*;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SurfaceSource<T> where T : AtomCreator {
    /// The temperature of the surface source, in Kelvin.
    pub temperature: f64,
    pub phantom: PhantomData<T>
}

impl<T> MaxwellBoltzmannSource for SurfaceSource<T> where T : AtomCreator {
    fn get_temperature(&self) -> f64 {
        self.temperature
    }
    fn get_v_dist_power(&self) -> f64 {
        2.0
    }
}

/// This system creates atoms from an oven source.
///
/// The oven points in the direction [Oven.direction].

pub fn create_atoms_on_surface_system<T>(
    surfaces: Query
            <(&SurfaceSource<T>, 
            &Cylinder, 
            &AtomNumberToEmit, 
            &Position,
            &PrecalculatedSpeciesInformation)>,
    velocity_cap: Res<VelocityCap>,
    mut commands: Commands,
) where T: AtomCreator + 'static {
    // obey velocity cap.
    let max_vel = velocity_cap.value;

    let mut rng = rand::rng();
    for (_, shape, number_to_emit, source_position, species) in surfaces.iter() {
        for _i in 0..number_to_emit.number {
            // Get random speed and mass.
            let (mass, speed) = species.generate_random_mass_v(&mut rng);
            if speed > max_vel {
                continue;
            }

            // generate a random position on the surface.
            let (position, normal) = shape.get_random_point_on_surface(&source_position.pos);

            // lambert cosine emission
            let direction = -normal.normalize();
            let random_dir = Vector3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
            )
            .normalize();
            let perp_a = direction.cross(&random_dir);
            let perp_b = direction.cross(&perp_a);

            let domain: bool = rng.random();
            let var: f64 = rng.random_range(0.0..1.0);
            let phi: f64 = rng.random_range(0.0..2.0 * std::f64::consts::PI);
            let theta: f64;
            if domain {
                theta = var.acos() / 2.0;
            } else {
                theta = var.asin() / 2.0 + std::f64::consts::PI / 4.0;
            }
            let emission_direction = theta.cos() * direction
                + theta.sin() * (perp_a * phi.cos() + perp_b * phi.sin());

            let velocity = speed * emission_direction;

            let new_atom = commands
                .spawn((
                    Position { pos: position },
                    Velocity { vel: velocity },
                    Force::default(),
                    Mass { value: mass },
                    Atom,
                    InitialVelocity { vel: velocity },
                    NewlyCreated,))
                .id();
            T::mutate(&mut commands, new_atom);
        }
    }
}
