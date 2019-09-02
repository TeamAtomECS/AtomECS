use crate::integrator::{Step, Timestep};
use specs::{
    Component, Entities, HashMapStorage, Join, NullStorage, Read, ReadExpect, ReadStorage, System,
    VecStorage, WriteExpect, WriteStorage,
};
pub struct OptEarly {
    /// how long the timestep should be increased
    pub timethreshold: f64,
    pub if_opt: bool,
    pub opt_finish: bool,
}

impl OptEarly {
    pub fn not_opt() -> OptEarly {
        OptEarly {
            timethreshold: 0.0,
            if_opt: true,
            opt_finish: true,
        }
    }
    pub fn new(timethreshold: f64) -> OptEarly {
        OptEarly {
            timethreshold,
            if_opt: false,
            opt_finish: false,
        }
    }
}

/// a system that increase the timestep at the begining of the simulation
/// usually, if the timethreshold is carefully chosen, the impact on accuracy is not noticable
pub struct OptEarlySystem;

impl<'a> System<'a> for OptEarlySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, OptEarly>,
        WriteExpect<'a, Timestep>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (ents, mut opt, mut timestep, step): Self::SystemData) {
        if !opt.if_opt {
            println!("timestep increased");
            timestep.delta = timestep.delta * 2.;
            opt.if_opt = true;
        }
        let time = timestep.delta * (step.n as f64);
        if time > opt.timethreshold && !opt.opt_finish {
            println!("timestep decrease at time:{}", time);
            timestep.delta = timestep.delta / 2.;
            opt.opt_finish = true;
        }
    }
}