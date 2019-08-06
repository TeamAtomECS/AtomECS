extern crate specs;
use crate::atom::*;
use crate::integrator::Step;

use specs::{Join, ReadExpect, ReadStorage, System};

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

pub struct FileOutputSystem {
    pub frequency: u64,
    pub file_name: String,
    writer: BufWriter<File>,
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

        let writer = BufWriter::new(file);

        FileOutputSystem {
            file_name: file_name,
            frequency: frequency,
            writer: writer,
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
            match write!(self.writer, "{}\n", ctr) {
                Err(why) => panic!("couldn't write to output: {}", why.description()),
                Ok(_) => (),
            }

            //Write (x,y,z) for each atom
            for (pos, _) in (&positions, &atoms).join() {
                match write!(
                    self.writer,
                    "{:.8},{:.8},{:.8}\n",
                    pos.pos.index(0),
                    pos.pos.index(1),
                    pos.pos.index(2)
                ) {
                    Err(why) => panic!("could not write to output: {}", why.description()),
                    Ok(_) => (),
                }
            }
        }
    }
}
