use crate::integrator::{Step, Timestep};
use specs::{
    Component, Entities, HashMapStorage, Join, NullStorage, Read, ReadExpect, ReadStorage, System,
    VecStorage, WriteExpect, WriteStorage,
};
pub struct OptEarly {
    /// how long the timestep should be increased
    pub timethreshold: f64,
    pub if_opt: bool,
}

impl Component for OptEarly {
    type Storage = HashMapStorage<Self>;
}

/// a system that increase the timestep at the begining of the simulation
/// usually, if the timethreshold is carefully chosen, the impact on accuracy is not noticable
pub struct OptEarlySystem;

impl<'a> System<'a> for OptEarlySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, OptEarly>,
        WriteExpect<'a, Timestep>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (ents, mut opt, mut timestep, step): Self::SystemData) {
        for (ent, mut opt) in (&ents, &mut opt).join() {
            if !opt.if_opt {
                println!("timestep increased");
                timestep.delta = timestep.delta * 2.;
                opt.if_opt = true;
            }
            let time = timestep.delta * (step.n as f64);
            if time > opt.timethreshold {
                println!("timestep decrease at time:{}", time);
                timestep.delta = timestep.delta / 2.;
                ents.delete(ent).expect("Could not delete entity");
            }
        }
    }
}