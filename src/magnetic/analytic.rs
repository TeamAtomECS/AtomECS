//! Support for analytically defined fields.

use super::MagneticFieldSampler;
use crate::{atom::Position, integrator::AtomECSBatchStrategy};
use bevy::prelude::*;
use nalgebra::{Matrix3, Vector3};

pub trait AnalyticField {
    /// Calculates the magnetic field.
    ///
    /// # Arguments
    ///
    /// `field_point`: position of the sampler, m
    ///
    /// `centre`: position of the quadrupole node, m
    fn get_field(&self, origin: Vector3<f64>, field_point: Vector3<f64>) -> Vector3<f64>;

    fn calculate_jacobian(&self) -> bool;
}

/// Adds contributions from a given field type to the [MagneticFieldSampler] components.
pub fn calculate_field_contributions<T>(
    fields_query: Query<(&Position, &T)>,
    mut samplers_query: Query<(&Position, &mut MagneticFieldSampler)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) where
    T: AnalyticField + Component,
{
    for (origin, field) in fields_query.iter() {
        samplers_query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(pos, mut sampler)| {
                // calculate field contribution
                sampler.field += field.get_field(origin.pos, pos.pos);

                if field.calculate_jacobian() {
                    //calculate jacobian
                    let mut jacobian = Matrix3::<f64>::zeros();
                    let delta = 1e-7; // Is there a better way to choose this number?
                                      // Strictly speaking to be accurate it depends on the length scale over which
                                      // the magnetic field changes
                    for i in 0..3 {
                        let mut pos_plus_dr = pos.pos;
                        let mut pos_minus_dr = pos.pos;
                        pos_plus_dr[i] += delta;
                        pos_minus_dr[i] -= delta;

                        let b_plus_dr = field.get_field(origin.pos, pos_plus_dr);
                        let b_minus_dr = field.get_field(origin.pos, pos_minus_dr);
                        let gradient = (b_plus_dr - b_minus_dr) / (2.0 * delta);
                        jacobian.set_column(i, &gradient);
                    }
                    sampler.jacobian += jacobian;
                }
            });
    }
}
