//! Writes output files containing atomic trajectories.
//!
//! To add file output to your simulation, add one or more `FileOutputPlugin`s, which determine
//! the component written to file and the output format used.

use crate::atom::Atom;
use crate::integrator::Step;
use bevy::prelude::*;
use nalgebra::Vector3;
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
#[derive(Resource)]
pub struct FileOutputResource<C: Component + Clone, F: Format<C, BufWriter<File>>, A = Atom> {
    /// Number of integration steps between each file output.
    pub interval: u64,
    /// The file name of the output file.
    pub file_name: String,
    /// Stream where output is written.
    stream: Option<BufWriter<File>>,
    atom_flag: PhantomData<A>,
    formatter: PhantomData<F>,
    /// The [Write](std::io::Write)able output stream.
    marker: PhantomData<C>,
}

pub struct FileOutputPlugin<C: Component + Clone, F: Format<C, BufWriter<File>>, A = Atom> {
    c_marker: PhantomData<C>,
    f_marker: PhantomData<F>,
    a_marker: PhantomData<A>,
    file_name: String,
    interval: u64,
}

impl<C, F, A> FileOutputPlugin<C, F, A>
where
    C: Component + Clone + Sync + Send,
    A: Component + Sync + Send,
    F: Format<C, BufWriter<File>> + Sync + Send,
{
    pub fn new(file_name: String, interval: u64) -> Self {
        FileOutputPlugin {
            c_marker: PhantomData,
            f_marker: PhantomData,
            a_marker: PhantomData,
            file_name,
            interval,
        }
    }
}

impl<C, F, A> Plugin for FileOutputPlugin<C, F, A>
where
    C: Component + Clone + Sync + Send + 'static,
    A: Component + Sync + Send + 'static,
    F: Format<C, BufWriter<File>> + Sync + Send + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(FileOutputResource::<C, F, A> {
            interval: self.interval,
            file_name: self.file_name.clone(),
            stream: None,
            atom_flag: PhantomData,
            formatter: PhantomData,
            marker: PhantomData,
        });
        app.add_systems(Update, update_writers::<C, F, A>);
    }
}

fn update_writers<C, F, A>(
    step: Res<Step>,
    mut outputter: ResMut<FileOutputResource<C, F, A>>,
    query: Query<(Entity, &C), With<A>>,
) where
    C: Component + Clone,
    A: Component,
    F: Format<C, BufWriter<File>> + Send + Sync + 'static,
{
    // if the stream is not opened, open it.
    if outputter.stream.is_none() {
        let path = Path::new(&outputter.file_name);
        let display = path.display();
        let file = match File::create(path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        let writer = BufWriter::new(file);
        outputter.stream = Option::Some(writer);
    }

    if step.n % outputter.interval == 0 {
        let atom_number = (query.into_iter()).count();
        F::write_frame_header(
            outputter.stream.as_mut().expect("File writer not open"),
            step.n,
            atom_number,
        )
        .expect("Could not write.");

        // write each entity
        for (ent, c) in query.iter() {
            F::write_atom(
                outputter.stream.as_mut().expect("File writer not open"),
                ent,
                c.clone(),
            )
            .expect("Could not write.");
        }
    }
}

// /// Creates a new [OutputSystem](struct.OutputSystem.html) to write per-entity [Component](specs::Component) data
// /// according to the specified [Format](struct.Format.html).
// ///
// /// The interval specifies how often, in integration steps, the file should be written.
// ///
// /// Only component data of entities associated with a component given by `A` is written down.
// ///
// /// For example, `new_with_filter::<Position, Text, Atom>("pos.txt", 10).
// fn new_with_filter<C, F, A>(
//     file_name: String,
//     interval: u64,
// ) -> OutputSystem<C, BufWriter<File>, F, A>

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
        writeln!(
            writer,
            "{:?},{:?}: {}",
            atom.generation(),
            atom.index(),
            data
        )?;
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
            atom.generation(),
            atom.index(),
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
        writer.write_u32::<Endianness>(atom.generation())?;
        writer.write_u32::<Endianness>(atom.index())?;
        for element in data.data() {
            writer.write_f64::<Endianness>(element)?;
        }
        Ok(())
    }
}
