//! Various optimization strategies to improve performance.

use crate::integrator::{Step, Timestep};
use specs::{ReadExpect, System, Write, WriteExpect};

/// A resource that configures the [LargerEarlyTimestepOptimizationSystem](struct.LargerEarlyTimestepOptimizationSystem.html)
pub struct LargerEarlyTimestepOptimization {
    /// The end time for the `EarlySimulation` stage. See [Timestep](struct.Timestep.html) for details of time units.
    pub early_time: f64,
    /// Factor by which the [Timestep](struct.Timestep.html) is increased by during the `EarlySimulation` stage.
    pub factor: f64,
    /// state of the optimization
    state: OptimizationState,
}

enum OptimizationState {
    StartOfSimulation,
    EarlySimulation,
    LateSimulation,
}

impl LargerEarlyTimestepOptimization {
    pub fn new(early_time: f64) -> LargerEarlyTimestepOptimization {
        LargerEarlyTimestepOptimization {
            early_time: early_time,
            factor: 2.0,
            state: OptimizationState::StartOfSimulation,
        }
    }
}

/// A system that modifies the [Timestep](struct.Timestep.html) duration as the simulation proceeds.
///
/// At the start of the simulation, the timestep is increased by the factor defined by
/// [LargerEarlyTimestepOptimization.factor](struct.LargerEarlyTimestepOptimization.html).
/// The timestep stays at this duration until the [LargerEarlyTimestepOptimization.early_time] is reached.
/// At this point, the timestep duration is returned to the original value.
/// 
/// This optimization allows the simulation to evolve faster duration the initial stages, where atoms
/// travel fast along straight paths.
pub struct LargerEarlyTimestepOptimizationSystem;

impl<'a> System<'a> for LargerEarlyTimestepOptimizationSystem {
    type SystemData = (
        Option<Write<'a, LargerEarlyTimestepOptimization>>,
        WriteExpect<'a, Timestep>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (mut optimization_options, mut timestep, step): Self::SystemData) {
        match optimization_options {
            None => return,
            Some(mut opts) => match &opts.state {
                OptimizationState::StartOfSimulation => {
                    timestep.delta = timestep.delta * 2.;
                    println!("timestep increased");
                    opts.state = OptimizationState::EarlySimulation;
                }

                OptimizationState::EarlySimulation => {
                    let time = timestep.delta * (step.n as f64);
                    if time > opts.early_time {
                        timestep.delta = timestep.delta / 2.;
                        println!("timestep decrease at time:{}", time);
                        opts.state = OptimizationState::LateSimulation;
                    }
                }

                OptimizationState::LateSimulation => {}
            },
        }
    }
}
