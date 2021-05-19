//pub mod atom;
pub mod atom;
pub mod dipole_beam;
pub mod dipole_force;
pub mod intensity_gradient;
pub mod transition_switcher;

extern crate specs;
use crate::initiate::NewlyCreated;
use specs::{DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadStorage, System, World};

pub const DIPOLE_BEAM_LIMIT: usize = 16;

/// Attaches components used for dipole force calculation to newly created atoms.
///
/// They are recognized as newly created if they are associated with
/// the `NewlyCreated` component.
pub struct AttachDipoleComponentsToNewlyCreatedAtomsSystem;

impl<'a> System<'a> for AttachDipoleComponentsToNewlyCreatedAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, NewlyCreated>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
        for (ent, _) in (&ent, &newly_created).join() {
            updater.insert(
                ent,
                intensity_gradient::LaserIntensityGradientSamplers {
                    contents: [intensity_gradient::LaserIntensityGradientSampler::default();
                        DIPOLE_BEAM_LIMIT],
                },
            );
        }
    }
}

/// Adds the systems required by the module to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the systems run.
pub fn add_systems_to_dispatch(builder: &mut DispatcherBuilder<'static, 'static>, deps: &[&str]) {
    builder.add(
        AttachDipoleComponentsToNewlyCreatedAtomsSystem,
        "attach_atom_dipole_components",
        deps,
    );
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
        intensity_gradient::SampleLaserIntensityGradientSystem,
        "sample_intensity_gradient",
        &["index_dipole_lights"],
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
