//! Support for different shapes.

use nalgebra::Vector3;
use rand;
use rand::Rng;
use bevy::prelude::*;

pub trait Volume {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool;
}

pub trait Surface {
    /// Returns (random point, normal) on the surface, uniformly distributed. The normal points outwards.
    fn get_random_point_on_surface(
        &self,
        surface_position: &Vector3<f64>,
    ) -> (Vector3<f64>, Vector3<f64>);
}

/// A cylindrical shape
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Cylinder {
    /// Radius of the cylindrical volume.
    pub radius: f64,
    /// Length of the cylindrical volume.
    pub length: f64,
    /// A normalised vector, aligned to the direction of the cylinder.
    pub direction: Vector3<f64>,
    /// A normalised vector, aligned perpendicular to the cylinder.
    pub perp_x: Vector3<f64>,
    /// A normalised vector, aligned perpendicular to the cylinder.
    pub perp_y: Vector3<f64>,
}

impl Cylinder {
    pub fn new(radius: f64, length: f64, direction: Vector3<f64>) -> Cylinder {
        let dir = Vector3::new(0.23, 1.2, 0.4563).normalize();
        let perp_x = direction.normalize().cross(&dir).normalize();
        let perp_y = direction.normalize().cross(&perp_x).normalize();
        Cylinder {
            radius,
            length,
            direction: direction.normalize(),
            perp_x,
            perp_y,
        }
    }
}

impl Volume for Cylinder {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool {
        let delta = volume_position - entity_position;
        let projection = delta.dot(&self.direction);

        if f64::abs(projection) > self.length / 2.0 {
            return false;
        }
        let orthogonal = delta - projection * self.direction;
        orthogonal.norm_squared() < self.radius.powi(2)
    }
}

impl Surface for Cylinder {
    fn get_random_point_on_surface(
        &self,
        surface_position: &Vector3<f64>,
    ) -> (Vector3<f64>, Vector3<f64>) {
        // Should we spawn a point on the ends or the sleeve?
        let mut rng = rand::rng();
        let spawn_on_ends = rng.random_range(0.0..1.0) < (self.radius / (self.length + self.radius));

        if spawn_on_ends {
            //pick a side
            let sign = match rng.random::<bool>() {
                true => 1.0,
                false => -1.0,
            };
            let angle = rng.random_range(0.0..2.0 * std::f64::consts::PI);
            let f: f64 = rng.random_range(0.0..1.0);
            let radius = self.radius * f.sqrt();
            let normal = sign * self.direction;
            let point = surface_position
                + self.perp_x * radius * angle.cos()
                + self.perp_y * radius * angle.sin()
                + normal * self.length / 2.0;
            (point, normal)
        } else {
            let angle = rng.random_range(0.0..2.0 * std::f64::consts::PI);
            let axial = rng.random_range(-self.length..self.length) / 2.0;
            let normal = self.perp_x * angle.cos() + self.perp_y * angle.sin();
            let point = surface_position + normal * self.radius + self.direction * axial;
            (point, normal)
        }
    }
}

/// A sphere.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Sphere {
    pub radius: f64,
}

impl Volume for Sphere {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool {
        let delta = entity_position - volume_position;
        delta.norm_squared() < self.radius.powi(2)
    }
}

impl Surface for Sphere {
    fn get_random_point_on_surface(
        &self,
        surface_position: &Vector3<f64>,
    ) -> (Vector3<f64>, Vector3<f64>) {
        let mut rng = rand::rng();

        let theta = rng.random_range(0.0..std::f64::consts::PI);
        let phi = rng.random_range(0.0..2.0 * std::f64::consts::PI);

        let normal = Vector3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
        let position = surface_position + self.radius * normal;
        (position, normal)
    }
}

/// A cuboid.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Cuboid {
    /// The dimension of the cuboid volume, from center to vertex (1,1,1).
    pub half_width: Vector3<f64>,
}

impl Volume for Cuboid {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool {
        let delta = entity_position - volume_position;
        delta[0].abs() < self.half_width[0]
            && delta[1].abs() < self.half_width[1]
            && delta[2].abs() < self.half_width[2]
    }
}

impl Surface for Cuboid {
    fn get_random_point_on_surface(
        &self,
        surface_position: &Vector3<f64>,
    ) -> (Vector3<f64>, Vector3<f64>) {
        let mut rng = rand::rng();

        let mut point = Vector3::new(
            rng.random_range(-self.half_width[0]..self.half_width[0]),
            rng.random_range(-self.half_width[1]..self.half_width[1]),
            rng.random_range(-self.half_width[2]..self.half_width[2]),
        );

        // move to a random edge
        let edge = rng.random_range(0..6);
        match edge {
            0 => point[0] = -self.half_width[0],
            1 => point[0] = self.half_width[0],
            2 => point[1] = -self.half_width[1],
            3 => point[1] = self.half_width[1],
            4 => point[2] = -self.half_width[2],
            5 => point[2] = self.half_width[2],
            _ => (),
        };

        let normal = match edge {
            0 => Vector3::new(-1.0, 0.0, 0.0),
            1 => Vector3::new(1.0, 0.0, 0.0),
            2 => Vector3::new(0.0, -1.0, 0.0),
            3 => Vector3::new(0.0, 1.0, 0.0),
            4 => Vector3::new(0.0, 0.0, -1.0),
            5 => Vector3::new(0.0, 0.0, 1.0),
            _ => Vector3::new(0.0, 0.0, 0.0),
        };
        let position = surface_position + point;
        (position, normal)
    }
}