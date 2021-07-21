//! Calculation and initialization of optical forces and quantities exerted on the atoms

pub mod cooling;
pub mod dipole_beam;
pub mod frame;
pub mod gaussian;
pub mod index;
pub mod intensity;
pub mod intensity_gradient;
pub mod sampler;

use crate::initiate::NewlyCreated;
use crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME;
use specs::prelude::*;

pub const BEAM_LIMIT: usize = 16;

/// Attaches components used for optical force calculation to newly created atoms.
///
/// They are recognized as newly created if they are associated with
/// the `NewlyCreated` component.
pub struct AttachLaserComponentsToNewlyCreatedAtomsSystem;

impl<'a> System<'a> for AttachLaserComponentsToNewlyCreatedAtomsSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, NewlyCreated>,
		Read<'a, LazyUpdate>,
	);

	fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
		for (ent, _) in (&ent, &newly_created).join() {
			updater.insert(
				ent,
				sampler::LaserSamplerMasks {
					contents: [sampler::LaserSamplerMask::default(); BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				intensity::LaserIntensitySamplers {
					contents: [intensity::LaserIntensitySampler::default(); BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				intensity_gradient::LaserIntensityGradientSamplers {
					contents: [intensity_gradient::LaserIntensityGradientSampler::default();
						BEAM_LIMIT],
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
		AttachLaserComponentsToNewlyCreatedAtomsSystem,
		"attach_laser_components",
		deps,
	);
	builder.add(
		cooling::AttachIndexToCoolingLightSystem,
		"attach_cooling_index",
		deps,
	);
	builder.add(index::IndexLasersSystem, "index_lasers", deps);
	builder.add(
		dipole_beam::AttachIndexToDipoleLightSystem,
		"attach_dipole_index",
		deps,
	);
	builder.add(
		sampler::InitialiseLaserSamplerMasksSystem,
		"initialise_laser_sampler_masks",
		deps,
	);
	builder.add(
		intensity::InitialiseLaserIntensitySamplersSystem,
		"initialise_laser_intensity",
		deps,
	);
	builder.add(
		sampler::FillLaserSamplerMasksSystem,
		"fill_laser_sampler_masks",
		&["index_lasers", "initialise_laser_sampler_masks"],
	);
	builder.add(
		intensity::SampleLaserIntensitySystem,
		"sample_laser_intensity",
		&[
			"index_lasers",
			"initialise_laser_intensity",
			INTEGRATE_POSITION_SYSTEM_NAME,
		],
	);
	builder.add(
		intensity_gradient::SampleGaussianLaserIntensityGradientSystem,
		"sample_intensity_gradient",
		&["index_lasers"],
	);
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<gaussian::GaussianBeam>();
	world.register::<gaussian::CircularMask>();
	world.register::<frame::Frame>();
	world.register::<dipole_beam::DipoleLight>();
}
