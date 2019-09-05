//! Writes output files containing atomic trajectories.

use crate::atom::*;
use crate::integrator::Step;
use specs::{Component, Entities, Join, ReadExpect, ReadStorage, System, Entity};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;

extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

/// A system that writes simulation data to file.
///
/// This system writes per-atom data `C` to a file at a defined interval.
/// The data type `C` must be a [Component](specs::Component) and implement the
/// [Clone](struct.Clone.html) trait.
pub struct OutputSystem<C:Component+Clone, W:Write, F: Format<C,W>> {
    /// Number of integration steps between each file output.
    interval: u64,
    /// The [Write](std::io::Write)able output stream.
    stream : W,
    formatter: PhantomData<F>,
    marker: PhantomData<C>
}

/// Creates a new [OutputSystem](struct.OutputSystem.html) to write per-atom [Component](specs::Component) data
/// according to the specified [Format](struct.Format.html). 
/// 
/// The interval specifies how often, in integration steps, the file should be written.
/// 
/// For example, `new::<Position,Text>("pos.txt", 10).
pub fn new<C, F>(file_name: String, interval: u64) -> OutputSystem<C, BufWriter<File>, F>
where C: Component+Clone, F: Format<C,BufWriter<File>> {
            let path = Path::new(&file_name);
        let display = path.display();
        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };
        let writer = BufWriter::new(file);
        OutputSystem { 
            interval: interval,
            stream: writer,
            formatter: PhantomData,
            marker: PhantomData
        }
}

impl<'a,C,W,F> System<'a> for OutputSystem<C, W, F>
where
    C: Component+Clone,
    W: Write,
    F: Format<C,W>
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, C>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, data, atoms, step): Self::SystemData) {
        if step.n % self.interval == 0 {
        let atom_number = (&atoms).join().count();
        F::write_frame_header(&mut self.stream, step.n, atom_number);

        // write each atom
        for (data, _, ent) in (&data, &atoms, &entities).join() {
                 F::write_atom(&mut self.stream, ent, data.clone());
            }
        }
    }
}

/// A trait implemented for each file output format.
pub trait Format<C,W> where C:Component+Clone, W:Write {
    /// Writes data indicating the start of a frame.
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize);
    
    /// Writes data associated with an atom.
    fn write_atom(writer: &mut W, atom: Entity, data: C);
}

/// Prints files in a [Format](struct.Format.html) that is human readable.
/// 
/// The output file is structured as follows. Each frame begins with the line
/// `step n atomNumber`, where `n` is the step number and `atomNumber` the number of
/// atoms to write to the file. This is followed by the `data : T` for each atom,
/// written to the file in the format `gen id: data`, where `gen` and `id` are the
/// [Entity](specs::Entity) generation and id, and data consists of the per-atom payload.
/// 
/// Components printed using text must implement the [Display](std::fmt::Display) trait.
pub struct Text { }
impl<C,W> Format<C,W> for Text where C:Component+Clone+Display, W:Write {
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize) {
        match write!(writer, "step {:?}, {:?}\n", step, atom_number) {
                Err(why) => panic!("Could not write to output: {}", why.description()),
                Ok(_) => (),
        };
    }

    fn write_atom(writer: &mut W, atom: Entity, data: C) {
                match write!(
                    writer,
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

type Endianness = LittleEndian;

pub trait BinaryConversion {
    fn data(&self) -> Vec<f64>;
    }

pub struct Binary {}
impl<C,W> Format<C,W> for Binary where C:Component+Clone+BinaryConversion, W:Write {
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize) {
            writer
                .write_u64::<Endianness>(step)
                .expect("Could not write to file.");
            writer
                .write_u64::<Endianness>(atom_number as u64)
                .expect("Could not write to file.");
    }

    fn write_atom(writer: &mut W, atom: Entity, data: C) {
        writer
                    .write_i32::<Endianness>(atom.gen().id())
                    .expect("Could not write to file.");
                writer
                    .write_u32::<Endianness>(atom.id())
                    .expect("Could not write to file.");
                for element in data.data() {
                    writer
                        .write_f64::<Endianness>(element)
                        .expect("Could not write to file.")
                }
    }
}
