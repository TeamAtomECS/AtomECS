//! Common atom components and systems.

//use crate::initiate::DeflagNewAtomsSystem;
//use crate::integrator::AddOldForceToNewAtomsSystem;
//use crate::output::file::BinaryConversion;
//use crate::output::file::XYZPosition;
//use crate::ramp::Lerp;
use bevy::prelude::*;
use nalgebra::{Vector3};

//use serde::{Deserialize, Serialize};
use std::fmt;

/// Position of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres)
#[derive(Clone, Component)]
pub struct Position {
    /// position in 3D in units of m
    pub pos: Vector3<f64>,
}
impl Default for Position {
    fn default() -> Self {
        Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?},{:?},{:?})", self.pos[0], self.pos[1], self.pos[2])
    }
}
// impl BinaryConversion for Position {
//     fn data(&self) -> Vec<f64> {
//         vec![self.pos[0], self.pos[1], self.pos[2]]
//     }
// }

/// Velocity of an entity in space, with respect to cartesian x,y,z axes.
///
/// SI units (metres/second)
#[derive(Clone, Copy, Component)]
pub struct Velocity {
    /// velocity vector in 3D in units of m/s
    pub vel: Vector3<f64>,
}
impl fmt::Display for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?},{:?},{:?})", self.vel[0], self.vel[1], self.vel[2])
    }
}
// impl BinaryConversion for Velocity {
//     fn data(&self) -> Vec<f64> {
//         vec![self.vel[0], self.vel[1], self.vel[2]]
//     }
// }

/// Initial velocity of an atom.
///
/// See [Velocity](struct.Velocity.html).
#[derive(Component)]
pub struct InitialVelocity {
    /// velocity vector in 3D in units of m/s
    pub vel: Vector3<f64>,
}

/// Force applied to an entity, with respect to cartesian x,y,z axes.
///
/// SI units (Newtons)
#[derive(Copy, Clone, Component)]
pub struct Force {
    /// force vector in 3D in units of N
    pub force: Vector3<f64>,
}
impl Default for Force {
    fn default() -> Self {
        Force {
            force: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

/// Inertial and Gravitational mass of an entity
///
/// Mass is specified in atom mass units (amu).
#[derive(Clone, Component)]
pub struct Mass {
    /// mass value in atom mass units
    pub value: f64,
}

/// Component that marks an entity as an [atom](struct.Atom.html).
/// This provides a simple way for systems to get only [atom](struct.Atom.html)s, even though non-atom entities may also share components, eg [position](struct.Position.html).
#[derive(Default, Component)]
pub struct Atom;
