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
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (positions, velocity, atoms, step): Self::SystemData) {
        // Write number of atoms
        if step.n % self.frequency == 0 {
            let mut ctr = 0;
            for _pos in (&positions, &atoms).join() {
                ctr = ctr + 1;
            }
            //match write!(self.writer, "{}\n", ctr) {
            //    Err(why) => panic!("couldn't write to output: {}", why.description()),
            //    Ok(_) => (),
            //}

            //Write (x,y,z) for each atom
            let mut content = vec![0.,0.,0.,0.,0.,0.,0.,0.,0.,0.,0.,0.];
            for (pos, vel, atom) in (&positions, &velocity, &atoms).join() {
                //println!("atom");
                content[(atom.index*2) as usize]=(pos.pos[2]);
                content[(atom.index*2+1) as usize]=(vel.vel[2]);

            }
            //println!("{:?}", content);
            if content.len() != 0 {
                match write!(
                    self.writer,
                    "{:.8},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8},{:.8}\n",
                    content[0],
                    content[1],
                    content[2],
                    content[3],
                    content[4],
                    content[5],
                    content[6],
                    content[7],
                    content[8],
                    content[9],
                    content[10],
                    content[11]
                ) {
                    Err(why) => panic!("could not write to output: {}", why.description()),
                    Ok(_) => (),
                }
            }
        }
    }
}
