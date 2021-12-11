//! Define magnetic fields using grids.

extern crate nalgebra;
use crate::atom::Position;
use crate::magnetic::MagneticFieldSampler;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};
extern crate serde;
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
#[derive(Serialize, Deserialize)]
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
        self.extent_cells[2] as i32
            * (self.extent_cells[1] as i32 * (cell_id[0] as i32) + (cell_id[1] as i32))
            + (cell_id[2] as i32)
    }

    pub fn get_field(&self, pos: &Vector3<f64>) -> Vector3<f64> {
        let index = self.position_to_grid_index(pos);
        self.grid[index as usize]
    }
}

impl Component for PrecalculatedMagneticFieldGrid {
    type Storage = HashMapStorage<Self>;
}

/// Samples from the MagneticFieldGrid at a `Position` and stores
/// result in `MagneticFieldSampler`
pub struct SampleMagneticGridSystem;
impl<'a> System<'a> for SampleMagneticGridSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, PrecalculatedMagneticFieldGrid>,
    );
    fn run(&mut self, (mut sampler, pos, grids): Self::SystemData) {
        for grid in (&grids).join() {
            for (pos, sampler) in (&pos, &mut sampler).join() {
                let field = grid.get_field(&pos.pos);
                sampler.field += field;
            }
        }
    }
}
