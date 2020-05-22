//! Support for different shapes.

use nalgebra::Vector3;
extern crate rand;
use rand::Rng;

trait Volume {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool;
}

trait Surface {
    /// Returns (random point, normal) on the surface, uniformly distributed. The normal points outwards.
    fn get_random_point_on_surface(&self, surface_position: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>);
}

/// A cylindrical shape
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

impl Volume for Cylinder {
    fn contains(&self, volume_position: &Vector3<f64>, entity_position: &Vector3<f64>) -> bool {
        let delta = volume_position - entity_position;
        let projection = delta.dot(&self.direction);

        if f64::abs(projection) > self.length / 2.0 {
            return false;
        }
        let orthogonal = delta - projection * self.direction;
        return orthogonal.norm_squared() < self.radius.powi(2);
    }
}

impl Surface for Cylinder {
    fn get_random_point_on_surface(&self, surface_position: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
        // Should we spawn a point on the ends or the sleeve?
        let mut rng = rand::thread_rng();
        let spawn_on_ends = rng.gen_range(0.0, 1.0) > self.length / self.radius;

        if spawn_on_ends
        {
            //pick a side
            let sign = match rng.gen::<(bool)>()
            {
                true => 1.0,
                false => -1.0
            };
            let angle = rng.gen_range(0.0, 2.0 * std::f64::consts::PI);
            let radius = rng.gen_range(0.0, &self.radius);
            let normal = sign * self.direction;
            let point = surface_position + self.perp_x * radius * angle.cos() + self.perp_y * radius * angle.sin() + normal * self.length / 2.0;
            return (point, normal);
        } else {
            let angle = rng.gen_range(0.0, 2.0 * std::f64::consts::PI);
            let axial = rng.gen_range(-self.length, self.length) / 2.0;
            let normal = self.perp_x * angle.cos() + self.perp_y * angle.sin();
            let point = surface_position + normal * self.radius + self.direction * axial;
            return (point, normal);
        }
    }
}