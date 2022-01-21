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

extern crate specs;
use specs::{Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System};

pub struct SurfaceSource<T> where T : AtomCreator {
    /// The temperature of the surface source, in Kelvin.
    pub temperature: f64,
    phantom: PhantomData<T>
}
impl<T> Component for SurfaceSource<T> where T : AtomCreator + 'static {
    type Storage = HashMapStorage<Self>;
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
#[derive(Default)]
pub struct CreateAtomsOnSurfaceSystem<T>(PhantomData<T>);
impl<'a, T> System<'a> for CreateAtomsOnSurfaceSystem<T> where T : AtomCreator + 'static {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, SurfaceSource<T>>,
        ReadStorage<'a, Cylinder>,
        ReadStorage<'a, AtomNumberToEmit>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, PrecalculatedSpeciesInformation>,
        Option<Read<'a, VelocityCap>>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            entities,
            surfaces,
            shapes,
            numbers_to_emit,
            source_positions,
            species,
            velocity_cap,
            updater,
        ): Self::SystemData,
    ) {
        // obey velocity cap.
        let max_vel = match velocity_cap {
            Some(cap) => cap.value,
            None => std::f64::MAX,
        };

        let mut rng = rand::thread_rng();
        for (_, shape, number_to_emit, source_position, species) in (
            &surfaces,
            &shapes,
            &numbers_to_emit,
            &source_positions,
            &species,
        )
            .join()
        {
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
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                )
                .normalize();
                let perp_a = direction.cross(&random_dir);
                let perp_b = direction.cross(&perp_a);

                let domain: bool = rng.gen();
                let var: f64 = rng.gen_range(0.0..1.0);
                let phi: f64 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                let theta: f64;
                if domain {
                    theta = var.acos() / 2.0;
                } else {
                    theta = var.asin() / 2.0 + std::f64::consts::PI / 4.0;
                }
                let emission_direction = theta.cos() * direction
                    + theta.sin() * (perp_a * phi.cos() + perp_b * phi.sin());

                let velocity = speed * emission_direction;

                let new_atom = entities.create();
                updater.insert(new_atom, Position { pos: position });
                updater.insert(new_atom, Velocity { vel: velocity });
                updater.insert(new_atom, Force::new());
                updater.insert(new_atom, Mass { value: mass });
                updater.insert(new_atom, Atom);
                updater.insert(new_atom, InitialVelocity { vel: velocity });
                updater.insert(new_atom, NewlyCreated);
                T::mutate(&updater, new_atom);
            }
        }
    }
}
