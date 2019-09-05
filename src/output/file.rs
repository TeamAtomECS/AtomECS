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

pub struct Text { }
impl<C,W> Format<C,W> for Text where C:Component+Clone+Display, W:Write {
    fn test(writer: &mut W, data: C)
    {
        write!(writer, "{}", data);
    }
}

/// A trait implemented by output formats.
pub trait Format<C,W> where C:Component+Clone, W:Write {
    fn test(writer: &mut W, data: C);
}

pub struct OutputSystem<C:Component+Clone, W:Write, F: Format<C,W>> {
    interval: u64,
    stream : W,
    formatter: PhantomData<F>,
    marker: PhantomData<C>
}

impl<C,W,F> OutputSystem<C,W,F> 
where C: Component+Clone,
W:Write,
F:Format<C,W>
{
    pub fn test(&mut self, data: C) {
        F::test(&mut self.stream, data);
        //self.stream.write(buf: &[u8])
    }
}

/// `let output = new<Position,Text>("pos.txt", 10)`
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
        //     self.write_frame_header(step.n, atom_number);

        //     // write each atom
        for (data, _, ent) in (&data, &atoms, &entities).join() {
        //         self.write_atom(ent, data.clone());

            }
        }
    }
}