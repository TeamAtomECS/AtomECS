//! Common atom components and systems.

use crate::output::file::BinaryConversion;
use crate::output::file::XYZPosition;
use crate::ramp::Lerp;
use nalgebra::Vector3;
use specs::prelude::*;

use serde::{Deserialize, Serialize};
use specs::{Component, NullStorage, System, VecStorage, World, WriteStorage};
use std::fmt;

/// Position of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres)
#[derive(Deserialize, Serialize, Clone, Lerp)]
pub struct Position {
    /// position in 3D in units of m
    pub pos: Vector3<f64>,
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl Position {
    pub fn new() -> Self {
        Position {
            /// position in 3D in units of m
            pos: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?},{:?},{:?})", self.pos[0], self.pos[1], self.pos[2])
    }
}
impl BinaryConversion for Position {
    fn data(&self) -> Vec<f64> {
        vec![self.pos[0], self.pos[1], self.pos[2]]
    }
}
impl XYZPosition for Position {
    fn pos(&self) -> Vector3<f64> {
        self.pos
    }
}

/// Velocity of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres/second)
#[derive(Clone, Copy, Serialize)]
pub struct Velocity {
    /// velocity vector in 3D in units of m/s
    pub vel: Vector3<f64>,
}
impl fmt::Display for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?},{:?},{:?})", self.vel[0], self.vel[1], self.vel[2])
    }
}
impl BinaryConversion for Velocity {
    fn data(&self) -> Vec<f64> {
        vec![self.vel[0], self.vel[1], self.vel[2]]
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

/// Initial velocity of an atom.
///
/// See [Velocity](struct.Velocity.html).
pub struct InitialVelocity {
    /// velocity vector in 3D in units of m/s
    pub vel: Vector3<f64>,
}
impl Component for InitialVelocity {
    type Storage = VecStorage<Self>;
}

/// Force applies to an entity, with respect to cartesian x,y,z axes.
///
/// SI units (Newtons)
#[derive(Copy, Clone, Serialize)]
pub struct Force {
    /// force vector in 3D in units of N
    pub force: Vector3<f64>,
}
impl Component for Force {
    type Storage = VecStorage<Self>;
}
impl Default for Force {
    fn default() -> Self {
        Self::new()
    }
}

impl Force {
    pub fn new() -> Self {
        Force {
            force: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

/// Inertial and Gravitational mass of an entity
///
/// Mass is specified in atom mass units (amu).
#[derive(Deserialize, Serialize, Clone)]
pub struct Mass {
    /// mass value in atom mass units
    pub value: f64,
}

impl Component for Mass {
    type Storage = VecStorage<Self>;
}

/// Component that marks an entity as an [atom](struct.Atom.html).
/// This provides a simple way for systems to get only [atom](struct.Atom.html)s, even though non-atom entities may also share components, eg [position](struct.Position.html).
#[derive(Default)]
pub struct Atom;

impl Component for Atom {
    type Storage = NullStorage<Self>;
}

/// A system that sets force to zero at the start of each simulation step.
pub struct ClearForceSystem;

impl<'a> System<'a> for ClearForceSystem {
    type SystemData = WriteStorage<'a, Force>;
    fn run(&mut self, mut force: Self::SystemData) {
        use rayon::prelude::*;

        (&mut force).par_join().for_each(|force| {
            force.force = Vector3::new(0.0, 0.0, 0.0);
        });
    }
}

/// Registers resources required by `atom_sources` to the ecs world.
pub fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Mass>();
    world.register::<Force>();
    world.register::<Atom>();
    world.register::<InitialVelocity>();
    world.register::<Velocity>();
}
