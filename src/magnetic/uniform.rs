extern crate specs;
extern crate nalgebra;
use super::MagneticFieldSampler;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};
use nalgebra::Vector3;

/// A component representing a uniform bias field, of the form `B = [ B_x, B_y, B_z ]`
pub struct UniformMagneticField {
    /// Vector field components with respect to the x,y,z cartesian axes, in units of Tesla.
    pub field: Vector3<f64>,
}

impl Component for UniformMagneticField {
    type Storage = HashMapStorage<Self>;
}

impl UniformMagneticField {
    /// Create a UniformMagneticField with components specified in units of Gauss.
    pub fn gauss(components: Vector3<f64>) -> UniformMagneticField {
        UniformMagneticField {
            field: components * 1.0e-4,
        }
    }

    /// Create a UniformMagneticField with components specified in units of Tesla.
    pub fn tesla(components: Vector3<f64>) -> UniformMagneticField {
        UniformMagneticField { field: components }
    }
}

/// Updates the values of magnetic field samplers to include uniform magnetic fields in the world.
pub struct UniformMagneticFieldSystem;

impl<'a> System<'a> for UniformMagneticFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, UniformMagneticField>,
    );
    fn run(&mut self, (mut sampler, fields): Self::SystemData) {
        for field in (&fields).join() {
            for mut sampler in (&mut sampler).join() {
                sampler.field = sampler.field + field.field;
            }
        }
    }
}