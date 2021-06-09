//pub mod atom;
pub mod atom;
pub mod dipole_force;
pub mod transition_switcher;

extern crate specs;
use specs::DispatcherBuilder;

/// Adds the systems required by the module to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the systems run.
pub fn add_systems_to_dispatch(builder: &mut DispatcherBuilder<'static, 'static>, _deps: &[&str]) {
    builder.add(
        dipole_force::ApplyDipoleForceSystem,
        "apply_dipole_force",
        &["sample_intensity_gradient"],
    );
}
