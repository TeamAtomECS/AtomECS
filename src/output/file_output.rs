//! Writes output files containing atomic trajectories.

use crate::atom::*;
use crate::integrator::Step;
use specs::{Component, Entities, Join, ReadExpect, ReadStorage, System};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::marker::PhantomData;
use std::path::Path;

/// A system that writes simulation data to file.
///
/// This system writes per-atom data `T` to a file at a defined interval.
/// The data type `T` must be a [Component](specs::Component) and implement the
/// [Display](std::fmt::Display) trait, which determines how the per-atom component is
/// presented in the output file.
///
/// The output file is structured as follows. Each frame begins with the line
/// `step n atomNumber`, where `n` is the step number and `atomNumber` the number of
/// atoms to write to the file. This is followed by the `data : T` for each atom,
/// written to the file in the format `gen id: data`, where `gen` and `id` are the
/// [Entity](specs::Entity) generation and id, and data consists of the per-atom payload.
pub struct FileOutputSystem<T: Component + Display> {
    /// The [FileOutputSystem](struct.FileOutputSystem.html) writes to file every time
    /// this number of steps are completed.
    pub interval: u64,

    /// The name of the file to write.
    pub file_name: String,

    writer: BufWriter<File>,

    phantom: std::marker::PhantomData<T>,
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
            match write!(self.writer, "step {:?}, {:?}\n", step.n, atom_number) {
                Err(why) => panic!("Could not write to output: {}", why.description()),
                Ok(_) => (),
            }

            //Write for each atom
            for (data, _, ent) in (&data, &atoms, &entities).join() {
                match write!(
                    self.writer,
                    "{:?},{:?}: {}\n",
                    ent.gen().id(),
                    ent.id(),
                    data
                ) {
                    Err(why) => panic!("Could not write to output: {}", why.description()),
                    Ok(_) => (),
                }
            }
        }
    }
}
