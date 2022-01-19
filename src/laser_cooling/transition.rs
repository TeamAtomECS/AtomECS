use crate::constant::BOHRMAG;
use specs::prelude::*;

/// Physical constants of an atomic transition used for laser cooling.
pub trait AtomicTransition {
    /// The dependence of the sigma_+ transition on magnetic fields.
    /// The sigma_+ transition is shifted by `mup * field.magnitude / h` Hz.
    /// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
    fn mup() -> f64;
    /// The dependence of the sigma_- transition on magnetic fields.
    /// The sigma_- transition is shifted by `mum * field.magnitude / h` Hz.
    /// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
    fn mum() -> f64;
    /// The dependence of the sigma_pi transition on magnetic fields.
    /// The sigma_pi transition is shifted by `muz * field.magnitude / h` Hz.
    /// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
    fn muz() -> f64;
    /// Frequency of the laser cooling transition, Hz.
    fn frequency() -> f64;
    /// Linewidth of the laser cooling transition, Hz
    fn linewidth() -> f64;
    /// Saturation intensity, in units of W/m^2.
    fn saturation_intensity() -> f64;
    /// Precalculated prefactor used in the determination of rate coefficients.
    fn rate_prefactor() -> f64;
    
    fn gamma() -> f64;


}

/// A transition which can be used as a component.
pub trait TransitionComponent : AtomicTransition + Component + Send + Sync + Default + Copy {}
impl<T: AtomicTransition + Component + Send + Sync + Default + Copy> TransitionComponent for T {}

// The bit below will be generated via a macro...

/// 461nm laser-cooling transition for 88Sr.
#[derive(Default, Copy, Clone)]
pub struct Strontium88_461;
impl AtomicTransition for Strontium88_461 {
    fn mup() -> f64 { BOHRMAG }
    fn mum() -> f64 { -BOHRMAG }
    fn muz() -> f64 { 0.0 }
    fn frequency() -> f64 { 650_759_219_088_937.0 }
    fn linewidth() -> f64 { 32e6 }
    fn saturation_intensity() -> f64 { 430.0 }
    fn rate_prefactor() -> f64 { Self::gamma().powi(3) / (Self::saturation_intensity() * 8.0) }
    fn gamma() -> f64 { Self::linewidth() * 2.0 * std::f64::consts::PI }
}
impl Component for Strontium88_461 {
    type Storage = VecStorage<Self>;
}