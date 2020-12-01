// This introduces a component "CentralCreator" of the source which spawns atoms with
// desired denstiy_distribution and velocity_distribution

extern crate nalgebra;
use rand::Rng;
extern crate specs;
use nalgebra::Vector3;

// Define some distributions that are necessary to custom-create the initial
// conditions of the atoms created

// Sample from this to get a position in space (could be a sphere or disc...)
#[derive(Copy, Clone)]
pub enum PositionDensityDistribution {
    UniformCuboidic { size: [f64; 3] },
    UniformSpheric { radius: f64 },
}

// Depending on your position, get a characteristic speed value.
// This is just to keep it general in case we would like to represent
// that particles in the middle are faster etc.
#[derive(Copy, Clone)]
pub enum SpatialSpeedDistribution {
    Uniform { speed: f64 },
    UniformCuboidic { speed: f64, size: [f64; 3] },
    UniformSpheric { speed: f64, radius: f64 },
}

// Depending on your characteristic speed value,
// a speed distribution is produced, sample from that to get the actual speed.
#[derive(Copy, Clone)]
pub enum SpeedDensityDistribution {
    // for UniformCentral: distribution like ____-----____ where width is width of -----
    // and the characteristic speed is center of support
    UniformCentral { width: f64 },
}

// Depending on your position, get a characteristic vector (for example pointing inwards)
#[derive(Copy, Clone)]
pub enum SpatialVectorDistribution {
    // no preferred direction, everywhere
    Uniform {},
}

// Depending on your characteristic vector, create a vector distribution.
// Sample from that to get the direction your velocity is pointing in.
#[derive(Copy, Clone)]
pub enum VectorDensityDistribution {
    // all directions equally probable. Practically ignores the characteristic vector.
    Uniform {},
}

/*
CentralCreator is the main structure of this script

It is designed in analogy to the Oven but without a builder (yet, might come later)

*/
pub struct CentralCreator {
    position_density_distribution: PositionDensityDistribution,
    spatial_speed_distribution: SpatialSpeedDistribution,
    speed_density_distribution: SpeedDensityDistribution,
    spatial_vector_distribution: SpatialVectorDistribution,
    vector_density_distribution: VectorDensityDistribution,
}

impl CentralCreator {
    // Create a new trivial, cubic central creator
    pub fn new_uniform_cubic(size_of_cube: f64, speed: f64) -> Self {
        Self {
            position_density_distribution: PositionDensityDistribution::UniformCuboidic {
                size: [size_of_cube, size_of_cube, size_of_cube],
            },
            spatial_speed_distribution: SpatialSpeedDistribution::Uniform { speed: speed },
            speed_density_distribution: SpeedDensityDistribution::UniformCentral { width: 0.0 },
            spatial_vector_distribution: SpatialVectorDistribution::Uniform {},
            vector_density_distribution: VectorDensityDistribution::Uniform {},
        }
    }

    // sample frome the oven and get random position and velocity vectors
    pub fn get_random_spawn_condition(&self) -> (Vector3<f64>, Vector3<f64>) {
        let mut rng = rand::thread_rng();

        let pos_vector = match self.position_density_distribution {
            PositionDensityDistribution::UniformCuboidic { size } => {
                let size = size.clone();
                let pos1 = rng.gen_range(-0.5 * size[0], 0.5 * size[0]);
                let pos2 = rng.gen_range(-0.5 * size[1], 0.5 * size[1]);
                let pos3 = rng.gen_range(-0.5 * size[2], 0.5 * size[2]);
                nalgebra::Vector3::new(pos1, pos2, pos3)
            }
            PositionDensityDistribution::UniformSpheric { radius: _ } => {
                // Not implemented!
                panic!("get_random_spawn_condition for PositionDensityDistribution::UniformSpheric not yet implemented!");
            }
        };

        let characteristic_speed: f64 = match self.spatial_speed_distribution {
            SpatialSpeedDistribution::Uniform { speed } => speed,
            SpatialSpeedDistribution::UniformCuboidic { speed: _, size: _ } => {
                // Not implemented!
                panic!("get_random_spawn_condition for SpatialSpeedDistribution::UniformCuboidic not yet implemented!");
            }
            SpatialSpeedDistribution::UniformSpheric {
                speed: _,
                radius: _,
            } => {
                // Not implemented!
                panic!("get_random_spawn_condition for SpatialSpeedDistribution::UniformSpheric not yet implemented!");
            }
        };

        let speed: f64 = match self.speed_density_distribution {
            SpeedDensityDistribution::UniformCentral { width } => {
                let min: f64 = (0.0f64).min(characteristic_speed - width);
                rng.gen_range(min, characteristic_speed + width)
            }
        };

        // so far this is ignored by the VectorDensityDistribution::Uniform {}
        // but this changes for mor complex VectorDensityDistributions
        let _characteristic_vector: Vector3<f64> = match self.spatial_vector_distribution {
            SpatialVectorDistribution::Uniform {} => Vector3::new(0.0, 0.0, 0.0),
        };

        let vector: Vector3<f64> = match self.vector_density_distribution {
            VectorDensityDistribution::Uniform {} => {
                let vec1 = rng.gen_range(-1.0, 1.0);
                let vec2 = rng.gen_range(-1.0, 1.0);
                let vec3 = rng.gen_range(-1.0, 1.0);
                (nalgebra::Vector3::new(vec1, vec2, vec3)).normalize()
            }
        };

        (pos_vector, speed * vector)
    }
}
