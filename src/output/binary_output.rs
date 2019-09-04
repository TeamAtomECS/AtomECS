//! Writes binary output files containing atomic trajectories.

extern crate byteorder;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::atom::*;
use crate::integrator::Step;
use specs::{Component, Entities, Join, ReadExpect, ReadStorage, System};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::marker::PhantomData;
use std::path::Path;

type Endianness = LittleEndian;

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
pub struct FileOutputSystem<T: Component + Binary> {
    /// The [FileOutputSystem](struct.FileOutputSystem.html) writes to file every time
    /// this number of steps are completed.
    pub interval: u64,

    /// The name of the file to write.
    pub file_name: String,

    writer: BufWriter<File>,

    phantom: std::marker::PhantomData<T>,
}

/// Trait used to output binary types from the simulation.
pub trait Binary {
    fn data(&self) -> Vec<f64>;
}

impl<T> FileOutputSystem<T>
where
    T: Component + Binary,
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
    T: Component + Binary,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, T>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, data, atoms, step): Self::SystemData) {
        // Write the payload for this frame.
        if step.n % self.interval == 0 {
            let atom_number = (&atoms).join().count();
            self.writer
                .write_u64::<Endianness>(step.n)
                .expect("Could not write to file.");
            self.writer
                .write_u64::<Endianness>(atom_number as u64)
                .expect("Could not write to file.");

            //Write for each atom
            for (data, _, ent) in (&data, &atoms, &entities).join() {
                self.writer
                    .write_i32::<Endianness>(ent.gen().id())
                    .expect("Could not write to file.");
                self.writer
                    .write_u32::<Endianness>(ent.id())
                    .expect("Could not write to file.");
                for element in data.data() {
                    self.writer
                        .write_f64::<Endianness>(element)
                        .expect("Could not write to file.")
                }
            }
        }
    }
}
