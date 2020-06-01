//! Simulate a 1D MOT.
//!
//! The 1D MOT is formed by counter-propagating laser beams along the z-axis.

extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::magnetic::grid::PrecalculatedMagneticFieldGrid;
use nalgebra::Vector3;
use specs::{Builder, World};
use std::fs::File;
extern crate serde;
extern crate serde_json;

fn main() {
    let grid = PrecalculatedMagneticFieldGrid {
        extent_spatial: Vector3::new(1.0,1.0,1.0),
        extent_cells: Vector3::new(1,1,1),
        position: Vector3::new(0.0,0.0,0.0),
        grid: vec!(Vector3::new(1.0,1.0,1.0))
        };
    let mut file = File::create("grid.json").expect("Cant open file");
    serde_json::to_writer(file, &grid);
}
