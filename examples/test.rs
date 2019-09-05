extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomInfo, Force, Mass, Position, Velocity};
use lib::output::file_output::{FileOutputSystem,Text};

fn main() {
      let test = FileOutputSystem::<Position,Text>::new("pos.txt".to_string(), 5);
      
}
