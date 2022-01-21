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
    /// The factor Gamma, equal to 2 pi times the linewidth.
    fn gamma() -> f64;
    /// Wavelength of the laser cooling transition, m.
    fn wavelength() -> f64;
}

/// A transition which can be used as a component.
pub trait TransitionComponent : AtomicTransition + Component + Send + Sync + Default + Copy {}
impl<T: AtomicTransition + Component + Send + Sync + Default + Copy> TransitionComponent for T {}

/// Generates a laser-cooling transition.
/// 
/// # Arguments:
/// * `transition_name`: name of the generated struct.
/// * `frequency`: frequency of the laser cooling transition, in Hz.
/// * `linewidth`: linewidth of the laser cooling transition, in Hz.
/// * `saturation_intensity`: Saturation intensity, in units of W/m^2.
/// * `mup`: shift of the sigma+ transition in magnetic field.
/// * `mum`: shift of the sigma- transition in magnetic field.
/// * `muz`: shift of the pi transition in magnetic field.
#[macro_export]
macro_rules! transition {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    (
        $transition_name:ident,
        $frequency: literal,
        $linewidth: literal,
        $saturation_intensity: literal,
        $mup: expr,
        $mum: expr,
        $muz: expr
    ) => {
        /// A laser cooling transition.
        #[derive(Copy, Clone, Default)]
        pub struct $transition_name;
        impl $crate::laser_cooling::transition::AtomicTransition for $transition_name {
            /// Frequency of the laser cooling transition, Hz.
            fn frequency() -> f64 { $frequency }
            /// Linewidth of the laser cooling transition, Hz.
            fn linewidth() -> f64 { $linewidth }
            /// Wavelength of the laser cooling transition, m.
            fn wavelength() -> f64 { crate::constant::C / $frequency }
            /// The dependence of the sigma_+ transition on magnetic fields.
            /// The sigma_+ transition is shifted by `mup * field.magnitude / h` Hz.
            /// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
            fn mup() -> f64 { $mup }
            /// The dependence of the sigma_- transition on magnetic fields.
            /// The sigma_- transition is shifted by `mum * field.magnitude / h` Hz.
            /// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
            fn mum() -> f64 { $mum }
            /// The dependence of the sigma_pi transition on magnetic fields.
            /// The sigma_pi transition is shifted by `muz * field.magnitude / h` Hz.
            /// The units of mup are of Energy per magnetic field, ie Joules/Tesla.
            fn muz() -> f64 { $muz }
            /// Saturation intensity, in units of W/m^2.
            fn saturation_intensity() -> f64 { $saturation_intensity }
            /// Precalculate prefactor used in the determination of rate coefficients.
            fn rate_prefactor() -> f64 { ($linewidth * 2.0 * std::f64::consts::PI).powi(3) / ($saturation_intensity * 8.0) }
            fn gamma() -> f64 { $linewidth * 2.0 * std::f64::consts::PI }
        }
        impl specs::Component for $transition_name {
            type Storage = specs::VecStorage<Self>;
        }
    };
}