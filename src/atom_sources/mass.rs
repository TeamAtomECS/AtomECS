use crate::atom::Mass;
extern crate rand;
use rand::Rng;
extern crate specs;

use serde::{Deserialize, Serialize};
use specs::{Component, HashMapStorage};

#[derive(Deserialize, Serialize, Clone)]
pub struct MassRatio {
    pub mass: f64,
    pub ratio: f64,
}

/// Describes the distribution of masses when atoms are created.
#[derive(Deserialize, Serialize, Clone)]
pub struct MassDistribution {
    pub distribution: Vec<MassRatio>,
    normalised: bool,
}
impl Component for MassDistribution {
    type Storage = HashMapStorage<Self>;
}
impl MassDistribution {
    /// Create a new distribution of masses
    pub fn new(distribution: Vec<MassRatio>) -> Self {
        let mut mass_dist = MassDistribution {
            distribution: distribution,
            normalised: false,
        };
        mass_dist.normalise();
        mass_dist
    }

    /// Normalise the distribution of masses so that the ratios add to one.
    fn normalise(&mut self) {
        let mut total = 0.;
        for mr in self.distribution.iter() {
            total = total + mr.ratio;
        }

        for mut mr in &mut self.distribution {
            mr.ratio = mr.ratio / total;
        }
        self.normalised = true
    }

    /// Randomly draw a mass from the distribution.
    pub fn draw_random_mass(&self) -> Mass {
        assert!(self.normalised);
        let mut level = 0.;
        let mut rng = rand::thread_rng();
        let luck = rng.gen_range(0.0, 1.0);
        let mut finalmass = 0.;
        for masspercent in self.distribution.iter() {
            level = level + masspercent.ratio;
            if level > luck {
                return Mass {
                    value: masspercent.mass,
                };
            }
            finalmass = masspercent.mass;
        }
        return Mass { value: finalmass };
    }
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
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
            total_ratio = total_ratio + mr.ratio;
        }

        assert_approx_eq!(total_ratio, 1., 0.0001);
    }
}
