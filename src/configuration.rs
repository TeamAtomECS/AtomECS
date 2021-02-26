//! Defines a struct that can be used to disable and enable certain features that
//! improve the physics of the simulation but cost runtime

#[derive(Debug)]
pub struct AtomECSConfiguration {
    /// If this is enabled, the number of actual photons will be drawn from a poisson distribution.
    ///
    /// Otherwise, the entries of `ActualPhotonsScatteredVector` will be identical with those of
    /// `ExpectedPhotonsScatteredVector`.
    pub scattering_fluctuations_option: Option,

    /// A resource that indicates that the simulation should apply random forces
    /// to simulate the random walk fluctuations due to spontaneous
    /// emission.
    pub emission_force_option: Option,
}

/// Per default, all Options are enabled such that the best physics simulation is done. The user must
/// deliberately switch off some feature to gain performance
impl Default for AtomECSConfiguration {
    fn default() -> Self {
        AtomECSConfiguration {
            scattering_fluctuations_option: Option::Enabled,
            emission_force_option: Option::Enabled,
        }
    }
}

/// Maybe with more options in the future?
#[derive(Debug)]
pub enum Option {
    Enabled,
    Disabled,
}
