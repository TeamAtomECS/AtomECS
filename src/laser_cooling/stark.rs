//! A module to treat the AC Stark shift on the transition frequency.
//!
//! Intense, far detuned optical radiation can be used to trap atoms, by modifying the energy
//! of the ground state through the AC Stark shift. When the AC Stark shift on the ground and
//! excited states are different, there is a shift in the resonant frequency of the transition
//! that connects them. This effect can become important when considering laser cooling in the
//! presence of dipole traps, and particularly for narrow-line width transitions.
//!
//! This module provides functionality to treat the AC stark shift on the resonant frequency.
//! A `CoolingTransitionACStarkShift` component should be added to individual atoms. This encodes
//! the frequency shift that results from an applied `DipoleLight` of known intensity. The result
//! is stored in the `CoolingTransitionACStarkShiftSampler`, which is automatically added.

use crate::dipole::DipoleLight;
use crate::laser::index::LaserIndex;
use crate::laser::intensity::LaserIntensitySamplers;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specs::prelude::*;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CoolingTransitionACStarkShift {
    /// The first-order shift in resonant frequency due to an intensity of light. Units of Hz m^2/W.
    pub susceptibility: f64,
}

impl Default for CoolingTransitionACStarkShift {
    fn default() -> Self {
        CoolingTransitionACStarkShift {
            susceptibility: 0.0,
        }
    }
}
impl Component for CoolingTransitionACStarkShift {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CoolingTransitionACStarkShiftSampler {
    /// The AC stark shift of the transition in units of Hz
    pub shift: f64,
}

impl Default for CoolingTransitionACStarkShiftSampler {
    fn default() -> Self {
        CoolingTransitionACStarkShiftSampler { shift: 0.0 }
    }
}
impl Component for CoolingTransitionACStarkShiftSampler {
    type Storage = VecStorage<Self>;
}

/// A system that calculates the AC Stark shift on a cooling transition.
pub struct CalculateCoolingTransitionACStarkShiftSystem;
impl<'a> System<'a> for CalculateCoolingTransitionACStarkShiftSystem {
    type SystemData = (
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, LaserIndex>,
        ReadStorage<'a, LaserIntensitySamplers>,
        ReadStorage<'a, CoolingTransitionACStarkShift>,
        WriteStorage<'a, CoolingTransitionACStarkShiftSampler>,
    );

    fn run(&mut self, (dipole, indices, intensities, shift, mut shift_samplers): Self::SystemData) {
        (&mut shift_samplers)
            .par_join()
            .for_each(|mut sampler| sampler.shift = 0.0);

        for (_dipole, index) in (&dipole, &indices).join() {
            (&mut shift_samplers, &shift, &intensities)
                .par_join()
                .for_each(|(mut sampler, shift, intensity)| {
                    sampler.shift =
                        shift.susceptibility * intensity.contents[index.index].intensity;
                });
        }
    }
}

/// A system that attaches `Samplers` to atoms which do not have them, but have the `CoolingTransitionACStarkShift` component.
pub struct AttachSamplersToAtomsSystem;
impl<'a> System<'a> for AttachSamplersToAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingTransitionACStarkShiftSampler>,
        ReadStorage<'a, CoolingTransitionACStarkShift>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, samplers, shifts, updater): Self::SystemData) {
        for (ent, _, _) in (&ent, &shifts, !&samplers).join() {
            updater.insert(ent, CoolingTransitionACStarkShiftSampler::default());
        }
    }
}
