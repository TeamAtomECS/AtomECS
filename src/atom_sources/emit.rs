//! Emission of atoms (over time)

extern crate nalgebra;
use crate::integrator::Timestep;
use rand;
use rand::Rng;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

/// Component which indicates the oven should emit a number of atoms per frame.
#[derive(Serialize, Deserialize, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct EmitNumberPerFrame {
    pub number: i32,
}

/// Component which indicates the oven should emit at a fixed average rate.
#[derive(Serialize, Deserialize, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct EmitFixedRate {
    pub rate: f64,
}

/// Component which sets future emission numbers to zero.
#[derive(Serialize, Deserialize, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct EmitOnce {}

/// The number of atoms the oven should emit in the current frame.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AtomNumberToEmit {
    pub number: i32,
}


/// Calculates the number of atoms to emit per frame for fixed atoms-per-timestep ovens
/// Not doing parallel iteration here, expect few ovens.
pub fn emit_number_per_frame_system(
    mut query: Query<(&EmitNumberPerFrame, &mut AtomNumberToEmit)>,
) {
    for (emit_number, mut number_to_emit) in query.iter_mut() {
        number_to_emit.number = emit_number.number;
    }
}

/// Calculates the number of atoms to emit each frame for sources with a fixed rate of emission.
///
/// There may be some random fluctuations in the numbers emitted each frame when the ratio of rate
/// and timestep duration is not an integer. The average rate will be correct.
pub fn emit_fixed_rate_system(
    mut query: Query<(&EmitFixedRate, &mut AtomNumberToEmit)>,
    timestep: Res<Timestep>,
) {
    let mut rng = rand::rng();
    for (rate, mut emit_numbers) in query.iter_mut() {
        let avg_number_to_emit = rate.rate * timestep.delta;
        let guaranteed_number = avg_number_to_emit.floor();
        let number: i32;
        if rng.random::<f64>() < avg_number_to_emit - guaranteed_number {
            number = guaranteed_number as i32 + 1;
        } else {
            number = guaranteed_number as i32;
        }
        emit_numbers.number = number;
    }
}


/// Sets the number of atoms to emit to zero for EmitOnce sources.
pub fn emit_once_system(
    mut query: Query<(&EmitOnce, &mut AtomNumberToEmit)>,
) {
    for (_, mut emit_numbers) in query.iter_mut() {
        emit_numbers.number = 0;
    }
}

#[cfg(test)]
pub mod tests {
    // These imports are actually needed! The compiler is getting confused and warning they are not.
    #[allow(unused_imports)]
    use super::*;
    // extern crate specs;
    #[allow(unused_imports)]
    // use specs::{Builder, Entity, RunNow, World};
    use bevy::prelude::*;
    extern crate nalgebra;
    #[allow(unused_imports)]
    use nalgebra::Vector3;

    #[test]
    fn test_fixed_rate_emitter() {
        let mut test = App::new();

        let time_delta = 1.0;
        test.insert_resource(Timestep { delta: time_delta });

        let rate = 3.3;

        let emitter = test.world_mut()
        .spawn((
            EmitFixedRate { rate },
            AtomNumberToEmit { number: 0 },
        )).id();

        let n = 1000;
        let mut total = 0;
        test.add_systems(Update, emit_fixed_rate_system);
        for _ in 1..n {
            test.update();
            let number = test.world().entity(emitter)
            .get::<AtomNumberToEmit>()
            .expect("Could not get entity")
            .number;
            assert!(number == 3 || number == 4);
            total += number;
        }
        assert!(total > (n as f64 * 0.9 * rate) as i32);
        assert!(total < (n as f64 * 1.1 * rate) as i32);
    }

    #[test]
    fn test_fixed_number_emitter() {
        let mut test = App::new();

        let number = 10;

        let emitter = test.world_mut()
            .spawn((
                EmitNumberPerFrame { number },
                AtomNumberToEmit { number: 0 },
            )).id();
        test.add_systems(Update, emit_number_per_frame_system);
        test.update();

        let emitted_number = test.world().entity(emitter)
            .get::<AtomNumberToEmit>()
            .expect("Could not get entity")
            .number;

        assert_eq!(number, emitted_number);
    }
}
