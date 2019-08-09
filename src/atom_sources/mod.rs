pub mod mass;
pub mod oven;

use specs::{DispatcherBuilder, World};

/// Adds the systems required by `atom_sources` to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the atom_sources systems run.
pub fn add_systems_to_dispatch(
    builder: DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) -> DispatcherBuilder<'static, 'static> {
    builder
        .with(
            oven::EmitNumberPerFrameSystem,
            "emit_number_per_frame",
            deps,
        )
        .with(
            oven::EmitFixedRateSystem,
            "emit_fixed_rate",
            &["emit_number_per_frame"]
        )
        .with(oven::OvenCreateAtomsSystem, "", &["emit_number_per_frame"])
}

/// Registers resources required by `atom_sources` to the ecs world.
pub fn register_components(world: &mut World) {
    world.register::<oven::Oven>();
    world.register::<mass::MassArchetype>();
    world.register::<oven::EmitFixedRate>();
    world.register::<oven::EmitNumberPerFrame>();
    world.register::<oven::AtomNumberToEmit>();
}
