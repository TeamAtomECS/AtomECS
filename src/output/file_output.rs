//! Writes output files containing atomic trajectories.

use crate::atom::*;
use crate::integrator::Step;
use std::fmt::Display;
use specs::{Component, Join, ReadExpect, ReadStorage, System, Entities};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::marker::PhantomData;
use std::path::Path;

/// A system that writes simulation data to file.
pub struct FileOutputSystem<T: Component + Display> {
    /// The [FileOutputSystem](struct.FileOutputSystem.html) writes to file every time
    /// this number of steps are completed.
    pub interval: u64,

    /// The name of the file to write.
    pub file_name: String,

    writer: BufWriter<File>,

    phantom: std::marker::PhantomData<T>,
}

/// Trait that indicates the type can be output to file.
pub trait Output {
    /// Converts the struct to a string representation.
    fn to_str(&self) -> str;
}

impl<T> FileOutputSystem<T>
where
    T: Component + Display,
{
    pub fn new(file_name: String, interval: u64) -> Self {
        let path = Path::new(&file_name);
        let display = path.display();
        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };
        let writer = BufWriter::new(file);
        FileOutputSystem {
            file_name: file_name,
            interval: interval,
            writer: writer,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> System<'a> for FileOutputSystem<T>
where
    T: Component + Display,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, T>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, data, atoms, step): Self::SystemData) {
            if step.n % self.interval == 0 {
                let atom_number = (&atoms).join().count();
                match write!(self.writer, "{:?} {:?}\n", step.n, atom_number) {
                    Err(why) => panic!("Could not write to output: {}", why.description()),
                    Ok(_) => (),
                }

                //Write for each atom
                for (data, _, ent) in (&data, &atoms, &entities).join() {
                    match write!(self.writer, "{:?},{:?}: {}", ent.gen().id(), ent.id(), data) {
                        Err(why) => panic!("Could not write to output: {}", why.description()),
                        Ok(_) => (),
                    }
                }
            }
    }
}
