extern crate specs;
use crate::atom::*;
use crate::integrator::{Step, Timestep};

use specs::{Join, ReadExpect, ReadStorage, System};

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct FileOutputSystem {
    pub frequency: u64,
    pub file_name: String,
    output_file: File,
}

impl FileOutputSystem {
    pub fn new(file_name: String, frequency: u64) -> FileOutputSystem {
        // Create a path to the desired file
        let path = Path::new(&file_name);
        let display = path.display();

        // Open the path, returns `io::Result<File>`
        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        FileOutputSystem {
            file_name: file_name,
            frequency: frequency,
            output_file: file,
        }
    }
}

impl<'a> System<'a> for FileOutputSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (positions, atoms, step): Self::SystemData) {
        // Write number of atoms
        if step.n % self.frequency == 0 {
        let mut ctr = 0;
        for _pos in (&positions, &atoms).join() {
            ctr = ctr + 1;
        }
        write!(self.output_file, "{}\n", ctr);

        //Write (x,y,z) for each atom
        let precision = 10;
        for (pos, _) in (&positions, &atoms).join() {
            write!(
                self.output_file,
                "{:.8},{:.8},{:.8}\n",
                pos.pos.index(0),
                pos.pos.index(1),
                pos.pos.index(2)
            );
        }
        }
    }
}
