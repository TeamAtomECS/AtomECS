//pub mod atom;
pub mod atom;
pub mod dipole_beam;
pub mod dipole_force;
pub mod transition_switcher;

extern crate specs;
use specs::{DispatcherBuilder, World};

pub const BEAM_LIMIT: usize = 16;

/// Adds the systems required by the module to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the systems run.
pub fn add_systems_to_dispatch(builder: &mut DispatcherBuilder<'static, 'static>, deps: &[&str]) {
    builder.add(
        dipole_beam::AttachIndexToDipoleLightSystem,
        "attach_dipole_index",
        deps,
    );
    builder.add(
        dipole_beam::IndexDipoleLightsSystem,
        "index_dipole_lights",
        &["attach_dipole_index"],
    );
    builder.add(
        dipole_force::ApplyDipoleForceSystem,
        "apply_dipole_force",
        &["sample_intensity_gradient"],
    );
}
pub fn register_components(world: &mut World) {
    world.register::<dipole_beam::DipoleLight>();
    world.register::<dipole_beam::DipoleLightIndex>();
}
