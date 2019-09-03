//! Writes diagnostic output to the console window.

use crate::atom::*;
use crate::integrator::{Step, Timestep};
use specs::{Join, ReadExpect, ReadStorage, System};

/// A system that writes diagnostic output to the console window.
pub struct ConsoleOutputSystem;

impl<'a> System<'a> for ConsoleOutputSystem {
    type SystemData = (
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
    );
    fn run(&mut self, (atom, step, timestep): Self::SystemData) {
        let _time = timestep.delta * step.n as f64;
        if step.n % 100 == 0 {
            let atom_number = (&atom).join().count();
            println!("Step {}: simulating {} atoms.", step.n, atom_number);
        }
    }
}
