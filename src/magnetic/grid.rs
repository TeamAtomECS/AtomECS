//! Define magnetic fields using grids.
use crate::magnetic::MagneticFieldSampler;
use crate::{atom::Position, integrator::AtomECSBatchStrategy};
use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

/// Defines a magnetic field using a grid-based representation.
///
/// The grid is ordered as a linear array, with elements ordered in priority z,y,x;
/// items with dz=1 are adjacent in memory.
///
/// # Fields
///
/// `extent_spatial`: Size of the grid, in units of m.
///
/// `position`: Position of the grid center, in units of m.
///
/// `extent_cells`: Size of the grid in cells, along the (x,y,z) axes.
///
/// `grid`: `Vec<Vector3<f64>>` containing the field at each grid cell.
#[derive(Serialize, Deserialize, Component)]
pub struct PrecalculatedMagneticFieldGrid {
    pub extent_spatial: Vector3<f64>,
    pub position: Vector3<f64>,
    pub extent_cells: Vector3<i32>,
    pub grid: Vec<Vector3<f64>>,
}

impl PrecalculatedMagneticFieldGrid {
    pub fn position_to_grid_index(&self, pos: &Vector3<f64>) -> i32 {
        let delta = pos - (self.position - self.extent_spatial / 2.0);
        let fraction = delta.component_div(&self.extent_spatial);
        // calculate cell ids
        let cell_id = Vector3::new(
            ((fraction[0] * self.extent_cells[0] as f64) as i32)
                .max(0)
                .min(self.extent_cells[0] - 1),
            ((fraction[1] * self.extent_cells[1] as f64) as i32)
                .max(0)
                .min(self.extent_cells[1] - 1),
            ((fraction[2] * self.extent_cells[2] as f64) as i32)
                .max(0)
                .min(self.extent_cells[2] - 1),
        );
        self.extent_cells[2] * (self.extent_cells[1] * cell_id[0] + cell_id[1]) + cell_id[2]
    }

    pub fn get_field(&self, pos: &Vector3<f64>) -> Vector3<f64> {
        let index = self.position_to_grid_index(pos);
        self.grid[index as usize]
    }
}

/// Samples from the [MagneticFieldGrid] at each [Position] and stores
/// results in the [MagneticFieldSampler]s
pub fn sample_magnetic_grids(
    grid_query: Query<&PrecalculatedMagneticFieldGrid>,
    mut sampler_query: Query<(&Position, &mut MagneticFieldSampler)>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    for grid in grid_query.iter() {
        sampler_query
            .par_iter_mut()
            .batching_strategy(batch_strategy.0.clone())
            .for_each(|(pos, mut sampler)| {
                let field = grid.get_field(&pos.pos);
                sampler.field += field;
            });
    }
}
