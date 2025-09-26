//! Masses and isotopes of atoms

use crate::atom::Mass;
use rand;
use rand::Rng;


use serde::{Deserialize, Serialize};

use bevy::prelude::*;

/// A [MassRatio](struct.MassRatio.html) describes the abundance of a given isotope.
#[derive(Deserialize, Serialize, Clone)]
pub struct MassRatio {
    /// The mass an atom will be created with. See [Mass](struct.Mass.html).
    pub mass: f64,
    /// The relative abundance of this mass.
    pub ratio: f64,
}

/// Describes the abundance of each mass.
///
/// When atoms are created, a random mass is drawn from the [MassDistribution](struct.MassDistribution.html) and assigned to the atom.
#[derive(Deserialize, Serialize, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct MassDistribution {
    pub distribution: Vec<MassRatio>,
    pub normalised: bool,
}

impl MassDistribution {
    /// Creates a new [MassDistribution](struct.MassDistribution.html), with the specified [MassRatio](struct.MassRatio.html)s.
    ///
    /// The created distribution will be normalised.
    pub fn new(distribution: Vec<MassRatio>) -> Self {
        let mut mass_dist = MassDistribution {
            distribution,
            normalised: false,
        };
        mass_dist.normalise();
        mass_dist
    }

    /// Normalises the distribution of masses so that the ratios add to one.
    pub fn normalise(&mut self) {
        let mut total = 0.;
        for mr in self.distribution.iter() {
            total += mr.ratio;
        }

        for mr in &mut self.distribution {
            mr.ratio /= total;
        }
        self.normalised = true
    }

    /// Randomly draw a mass from the distribution.
    pub fn draw_random_mass(&self) -> Mass {
        assert!(self.normalised);
        let mut level = 0.;
        let mut rng = rand::rng();
        let luck = rng.random_range(0.0..1.0);
        let mut finalmass = 0.;
        for masspercent in self.distribution.iter() {
            level += masspercent.ratio;
            if level > luck {
                return Mass {
                    value: masspercent.mass,
                };
            }
            finalmass = masspercent.mass;
        }
        Mass { value: finalmass }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_mass_distribution_normalised() {
        let mass_distribution = MassDistribution::new(vec![
            MassRatio {
                mass: 1.0,
                ratio: 10.0,
            },
            MassRatio {
                mass: 2.0,
                ratio: 1.0,
            },
        ]);

        let mut total_ratio = 0.0;
        for mr in &mass_distribution.distribution {
            total_ratio += mr.ratio;
        }

        assert_approx_eq!(total_ratio, 1., 0.0001);
    }
}
