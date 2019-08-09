extern crate nalgebra;
extern crate rand;
use crate::integrator::Timestep;
use rand::Rng;

extern crate specs;
use serde::{Deserialize, Serialize};

use specs::{Component, HashMapStorage, Join, ReadExpect, ReadStorage, System, WriteStorage};

/// Component which indicates the oven should emit a number of atoms per frame.
#[derive(Serialize, Deserialize, Clone)]
pub struct EmitNumberPerFrame {
    pub number: i32,
}
impl Component for EmitNumberPerFrame {
    type Storage = HashMapStorage<Self>;
}

/// Component which indicates the oven should emit at a fixed average rate.
#[derive(Serialize, Deserialize, Clone)]
pub struct EmitFixedRate {
    pub rate: f64,
}
impl Component for EmitFixedRate {
    type Storage = HashMapStorage<Self>;
}

/// The number of atoms the oven should emit in the current frame.
pub struct AtomNumberToEmit {
    pub number: i32,
}
impl Component for AtomNumberToEmit {
    type Storage = HashMapStorage<Self>;
}

/// Calculates the number of atoms to emit per frame for fixed atoms-per-timestep ovens
pub struct EmitNumberPerFrameSystem;
impl<'a> System<'a> for EmitNumberPerFrameSystem {
    type SystemData = (
        ReadStorage<'a, EmitNumberPerFrame>,
        WriteStorage<'a, AtomNumberToEmit>,
    );

    fn run(&mut self, (emit_numbers, mut numbers_to_emit): Self::SystemData) {
        for (emit_number, mut number_to_emit) in (&emit_numbers, &mut numbers_to_emit).join() {
            number_to_emit.number = emit_number.number;
        }
    }
}

/// Calculates the number of atoms to emit each frame for sources with a fixed rate of emission.
///
/// There may be some random fluctuations in the numbers emitted each frame when the ratio of rate
/// and timestep duration is not an integer. The average rate will be correct.
pub struct EmitFixedRateSystem;
impl<'a> System<'a> for EmitFixedRateSystem {
    type SystemData = (
        ReadStorage<'a, EmitFixedRate>,
        ReadExpect<'a, Timestep>,
        WriteStorage<'a, AtomNumberToEmit>,
    );

    fn run(&mut self, (rates, timestep, mut emit_numbers): Self::SystemData) {
        let mut rng = rand::thread_rng();
        for (rate, mut emit_numbers) in (&rates, &mut emit_numbers).join() {
            let avg_number_to_emit = rate.rate * timestep.delta;
            let guaranteed_number = avg_number_to_emit.floor();
            let number: i32;
            if rng.next_f64() < avg_number_to_emit - guaranteed_number {
                number = guaranteed_number as i32 + 1;
            } else {
                number = guaranteed_number as i32;
            }
            emit_numbers.number = number;
        }
    }
}
