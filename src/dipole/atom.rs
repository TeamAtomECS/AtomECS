use crate::atom::Kind;
use crate::constant::C;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

/// Component which holds information about the physical properties of the main
/// transition that is relevant for dipole cooling. Similar to `atom::AtomicTransition`.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct AtomicDipoleTransition {
    /// Frequency of the laser cooling transition, Hz.
    pub frequency: f64,
    /// Linewidth of the laser cooling transition, Hz
    pub linewidth: f64,
    /// Nametag for match control operators to identify later on
    pub kind: Kind,
}

impl Component for AtomicDipoleTransition {
    type Storage = VecStorage<Self>;
}

impl AtomicDipoleTransition {
    /// Creates an `AtomicDipoleTransition` component populated with parameters for Rubidium.
    /// The parameters are taken from Daniel Steck's Data sheet on Rubidium-87.
    pub fn rubidium() -> Self {
        AtomicDipoleTransition {
            frequency: C / 780.0e-9,
            linewidth: 6.065e6, // [Steck, Rubidium87]
            kind: Kind::Rubidium,
        }
    }

    /// Creates an `AtomicDipoleTransition` component populated with parameters for Strontium.
    /// The parameters are taken from doi:10.1103/PhysRevA.97.039901 [Nosske 2017].
    pub fn strontium() -> Self {
        AtomicDipoleTransition {
            frequency: 650_759_219_088_937.,
            linewidth: 32e6, // [Nosske2017]
            kind: Kind::Strontium,
        }
    }

    /// Creates an `AtomicDipoleTransition` component populated with parameters for red Strontium transition.
    /// The parameters are taken from NIST, doi:10.1063/1.344917 and Schreck2013.
    pub fn strontium_red() -> Self {
        AtomicDipoleTransition {
            frequency: 434_829_121_311_000., // NIST, doi:10.1063/1.344917
            linewidth: 7_400.,               // [Schreck2013]
            kind: Kind::StrontiumRed,
        }
    }

    /// Creates an `AtomicDipoleTransition` component populated with parameters for Erbium.
    pub fn erbium() -> Self {
        AtomicDipoleTransition {
            frequency: 5.142e14,
            linewidth: 190e3,
            kind: Kind::Erbium,
        }
    }
    /// Creates an `AtomicDipoleTransition` component populated with parameters for Erbium 401 .
    pub fn erbium_401() -> Self {
        AtomicDipoleTransition {
            frequency: 7.476e14,
            linewidth: 30e6,
            kind: Kind::Erbium401,
        }
    }

    /// calculates the natural linewidth in units of rad/s
    pub fn gamma(&self) -> f64 {
        self.linewidth * 2.0 * std::f64::consts::PI
    }
}
