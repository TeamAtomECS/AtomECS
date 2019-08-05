extern crate specs;
use crate::atom::*;
use crate::integrator::{Step, Timestep};

use specs::{Join, ReadExpect, ReadStorage, System};

pub struct PrintOutputSytem;

impl<'a> System<'a> for PrintOutputSytem {
    // print the output (whatever you want) to the console
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Atom>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
    );
    fn run(&mut self, (pos, vel, atom, step, timestep): Self::SystemData) {
        let _time = timestep.delta * step.n as f64;
        for (vel, pos, _) in (&vel, &pos, &atom).join() {
            if step.n % 100 == 0 {
                println!(
                    "step {}: position{:?},velocity{:?},",
                    step.n, pos.pos, vel.vel
                );
            }
        }
    }
}
