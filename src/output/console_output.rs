extern crate specs;
use crate::atom::*;

use crate::constant;
use crate::integrator::{Step, Timestep};
use specs::{Join, ReadExpect, ReadStorage, System};

pub struct ConsoleOutputSystem;

impl<'a> System<'a> for ConsoleOutputSystem {
    // print the output (whatever you want) to the console
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, Force>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
    );
    fn run(&mut self, (pos, vel, atom, force, step, timestep): Self::SystemData) {
        let _time = timestep.delta * step.n as f64;
        let mut atom_number = 0;
        if step.n % 1 == 0 {
            for (_vel, _pos, _, force) in (&vel, &pos, &atom, &force).join() {
                atom_number = atom_number + 1;
                println!(
                    "time: {}position: {:?}, velocity {:?},force{:?}",
                    _time,
                    _pos.pos,
                    _vel.vel,
                    force.force / constant::AMU
                );
            }
            println!("Step {}: atom_number={}", step.n, atom_number);
        }
    }
}
