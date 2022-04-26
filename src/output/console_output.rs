//! Writes diagnostic output to the console window.

use crate::atom::*;
use crate::integrator::{Step};
use bevy::prelude::*;

/// A system that writes diagnostic output to the console window.
pub fn console_output(
    step: Res<Step>,
    query: Query<&Atom>
) {
    if step.n % 100 == 0 {
        let atom_number = query.iter().count();
        println!("Step {}: simulating {} atoms.", step.n, atom_number);
    }
}