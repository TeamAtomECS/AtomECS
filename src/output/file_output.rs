//! Writes output files containing atomic trajectories.

use crate::atom::*;
use crate::integrator::Step;
use specs::{Component, Entities, Join, ReadExpect, ReadStorage, System, Entity};
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
pub struct FileOutputSystem<C:Component+Clone,W:Output> {
    /// The [FileOutputSystem](struct.FileOutputSystem.html) writes to file every time
    /// this number of steps are completed.
    pub interval: u64,

    /// The name of the file to write.
    pub file_name: String,

    writer: BufWriter<File>,

    phantom: std::marker::PhantomData<C>,
    output: std::marker::PhantomData<W>
}

impl<C,W> FileOutputSystem<C,W>
where
    C: Component+Clone,
    W: Output
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
            output: PhantomData
        }
    }
}

/// A system that does something periodically
trait PeriodicSystem {
    /// Number of simulation steps between action.
    fn get_interval(&self) -> u64;
}
impl<C,W> PeriodicSystem for FileOutputSystem<C,W>
where C:Component+Clone,W:Output
{
    fn get_interval(&self) -> u64 { self.interval }
}

/// A trait for structs that write atomic trajectories.
trait AtomWriterSystem : PeriodicSystem {
    type DataComponent : Component + Clone;

    /// Writes data indicating the start of a frame.
    fn write_frame_header(&mut self, step: u64, atom_number: usize);
    
    /// Writes data associated with an atom.
    fn write_atom(&mut self, atom: Entity, data: Self::DataComponent);
}

/// System implementation for all [AtomWriterSystem](struct.AtomWriterSystem.html)s.
/// 
/// Each time an interval number of steps elapses, the system writes a frame.
/// The frame consists of a header, followed by per-atom data.
impl<'a,C> System<'a> for AtomWriterSystem<DataComponent=C>
where
    C: Component+Clone
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, C>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, data, atoms, step): Self::SystemData) {
        if step.n % self.get_interval() == 0 {
            let atom_number = (&atoms).join().count();
            self.write_frame_header(step.n, atom_number);

            // write each atom
            for (data, _, ent) in (&data, &atoms, &entities).join() {
                self.write_atom(ent, data.clone());
            }
        }
    }
}

/// Indicates binary output.
/// 
/// See [FileOutputSystem](struct.FileOutputSystem.html)
pub struct Binary { }

/// Indicates text output.
/// 
/// See [FileOutputSystem](struct.FileOutputSystem.html)
pub struct Text { }

pub trait Output {}
impl Output for Text {}
impl Output for Binary {}

impl<C> AtomWriterSystem for FileOutputSystem<C,Text>
where C: Component+Clone+Display
{
    type DataComponent=C;

    fn write_frame_header(&mut self, step: u64, atom_number: usize)
    {
        match write!(self.writer, "step {:?}, {:?}\n", step, atom_number) {
                Err(why) => panic!("Could not write to output: {}", why.description()),
                Ok(_) => (),
        };
    }

    fn write_atom(&mut self, atom: Entity, data: C)
    {
        match write!(
                    self.writer,
                    "{:?},{:?}: {}\n",
                    atom.gen().id(),
                    atom.id(),
                    data
                ) {
                    Err(why) => panic!("Could not write to output: {}", why.description()),
                    Ok(_) => (),
                }
    }
}