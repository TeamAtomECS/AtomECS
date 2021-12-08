//! Writes output files containing atomic trajectories.
use crate::atom::Atom;
use crate::integrator::Step;
use nalgebra::Vector3;
use specs::{Component, Entities, Entity, Join, ReadExpect, ReadStorage, System};
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;

extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

/// A system that writes simulation data to file.
///
/// This system writes data `C` of entities associated with `A` to a file at a defined interval.
/// The data type `C` must be a [Component](specs::Component) and implement the
/// [Clone](struct.Clone.html) trait.
pub struct OutputSystem<C: Component + Clone, W: Write, F: Format<C, W>, A = Atom> {
    /// Number of integration steps between each file output.
    interval: u64,
    /// The [Write](std::io::Write)able output stream.
    atom_flag: PhantomData<A>,
    stream: W,
    formatter: PhantomData<F>,
    marker: PhantomData<C>,
}

/// Creates a new [OutputSystem](struct.OutputSystem.html) to write per-atom [Component](specs::Component) data
/// according to the specified [Format](struct.Format.html).
///
/// The interval specifies how often, in integration steps, the file should be written.
///
/// Only component data of entities associated with `Atom` is written down.
///
/// For example, `new::<Position, Text>("pos.txt", 10).
pub fn new<C, F>(file_name: String, interval: u64) -> OutputSystem<C, BufWriter<File>, F, Atom>
where
    C: Component + Clone,
    F: Format<C, BufWriter<File>>,
{
    new_with_filter::<C, F, Atom>(file_name, interval)
}

/// Creates a new [OutputSystem](struct.OutputSystem.html) to write per-entity [Component](specs::Component) data
/// according to the specified [Format](struct.Format.html).
///
/// The interval specifies how often, in integration steps, the file should be written.
///
/// Only component data of entities associated with a component given by `A` is written down.
///
/// For example, `new_with_filter::<Position, Text, Atom>("pos.txt", 10).
pub fn new_with_filter<C, F, A>(
    file_name: String,
    interval: u64,
) -> OutputSystem<C, BufWriter<File>, F, A>
where
    C: Component + Clone,
    A: Component,
    F: Format<C, BufWriter<File>>,
{
    let path = Path::new(&file_name);
    let display = path.display();
    let file = match File::create(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let writer = BufWriter::new(file);
    OutputSystem {
        interval,
        atom_flag: PhantomData,
        stream: writer,
        formatter: PhantomData,
        marker: PhantomData,
    }
}

impl<'a, C, A, W, F> System<'a> for OutputSystem<C, W, F, A>
where
    C: Component + Clone,
    A: Component,
    W: Write,
    F: Format<C, W>,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, C>,
        ReadStorage<'a, A>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, data, atom_flags, step): Self::SystemData) {
        if step.n % self.interval == 0 {
            let atom_number = (&atom_flags).join().count();
            F::write_frame_header(&mut self.stream, step.n, atom_number).expect("Could not write.");

            // write each entity
            for (data, _, ent) in (&data, &atom_flags, &entities).join() {
                F::write_atom(&mut self.stream, ent, data.clone()).expect("Could not write.");
            }
        }
    }
}

/// A trait implemented for each file output format.
pub trait Format<C, W>
where
    C: Component + Clone,
    W: Write,
{
    /// Writes data indicating the start of a frame.
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize) -> Result<(), io::Error>;
    /// Writes data associated with an atom.
    fn write_atom(writer: &mut W, atom: Entity, data: C) -> Result<(), io::Error>;
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
pub struct Text {}
impl<C, W> Format<C, W> for Text
where
    C: Component + Clone + Display,
    W: Write,
{
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize) -> Result<(), io::Error> {
        writeln!(writer, "step-{:?}, {:?}", step, atom_number)?;
        Ok(())
    }

    fn write_atom(writer: &mut W, atom: Entity, data: C) -> Result<(), io::Error> {
        writeln!(writer, "{:?},{:?}: {}", atom.gen().id(), atom.id(), data)?;
        Ok(())
    }
}

pub struct SerdeJson {}
impl<C, W> Format<C, W> for SerdeJson
where
    C: Component + serde::Serialize + Clone,
    W: Write,
{
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize) -> Result<(), io::Error> {
        writeln!(writer, "step-{:?}, {:?}", step, atom_number)?;
        Ok(())
    }

    fn write_atom(writer: &mut W, atom: Entity, data: C) -> Result<(), io::Error> {
        let serialized = serde_json::to_string(&data).unwrap();
        writeln!(
            writer,
            "{:?},{:?}, {}",
            atom.gen().id(),
            atom.id(),
            serialized
        )?;
        Ok(())
    }
}
pub trait XYZPosition {
    fn pos(&self) -> Vector3<f64>;
}

pub struct XYZ {}
impl<C, W> Format<C, W> for XYZ
where
    C: Component + Clone + XYZPosition,
    W: Write,
{
    fn write_frame_header(writer: &mut W, _step: u64, atom_number: usize) -> Result<(), io::Error> {
        write!(writer, "{:?}\n\n", atom_number)?;
        Ok(())
    }

    fn write_atom(writer: &mut W, _atom: Entity, data: C) -> Result<(), io::Error> {
        // the scale factor is 20000
        let position = 20000.0 * data.pos();
        writeln!(
            writer,
            "H\t{}\t{}\t{}",
            position[0], position[1], position[2]
        )?;
        Ok(())
    }
}

type Endianness = LittleEndian;

pub trait BinaryConversion {
    fn data(&self) -> Vec<f64>;
}

pub struct Binary {}
impl<C, W> Format<C, W> for Binary
where
    C: Component + Clone + BinaryConversion,
    W: Write,
{
    fn write_frame_header(writer: &mut W, step: u64, atom_number: usize) -> Result<(), io::Error> {
        writer.write_u64::<Endianness>(step)?;
        writer.write_u64::<Endianness>(atom_number as u64)?;
        Ok(())
    }

    fn write_atom(writer: &mut W, atom: Entity, data: C) -> Result<(), io::Error> {
        writer.write_i32::<Endianness>(atom.gen().id())?;
        writer.write_u32::<Endianness>(atom.id())?;
        for element in data.data() {
            writer.write_f64::<Endianness>(element)?;
        }
        Ok(())
    }
}
