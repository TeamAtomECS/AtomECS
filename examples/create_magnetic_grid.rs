//! Create a magnetic grid file. This is really just used to show you what format the program expects the file to be in.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::magnetic::grid::PrecalculatedMagneticFieldGrid;
use nalgebra::Vector3;
use std::fs::File;
extern crate serde;
extern crate serde_json;

fn main() {
    let grid = PrecalculatedMagneticFieldGrid {
        extent_spatial: Vector3::new(1.0, 1.0, 1.0),
        extent_cells: Vector3::new(1, 1, 1),
        position: Vector3::new(0.0, 0.0, 0.0),
        grid: vec![Vector3::new(1.0, 1.0, 1.0)],
    };
    let file = File::create("grid.json").expect("Cant open file");
    serde_json::to_writer(file, &grid).expect("Could not serialize grid");
}
