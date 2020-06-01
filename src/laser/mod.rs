pub mod cooling;
pub mod doppler;
pub mod force;
pub mod gaussian;
pub mod repump;
pub mod sampler;

extern crate specs;
use crate::initiate::NewlyCreated;
use crate::laser::force::NumberKick;
use specs::{DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadStorage, System, World};

/// Attachs components used for optical force calculation to newly created atoms.
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
				sampler::LaserSamplers {
					contents: Vec::new(),
				},
			);
			updater.insert(ent, NumberKick { value: 0.0 });
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
pub fn add_systems_to_dispatch(
	builder: DispatcherBuilder<'static, 'static>,
	deps: &[&str],
) -> DispatcherBuilder<'static, 'static> {
	builder
		.with(
			AttachLaserComponentsToNewlyCreatedAtomsSystem,
			"attach_atom_laser_components",
			deps,
		)
		.with(
			cooling::AttachIndexToCoolingLightSystem,
			"attach_cooling_index",
			deps,
		)
		.with(
			cooling::IndexCoolingLightsSystem,
			"index_cooling_lights",
			&["attach_cooling_index"],
		)
		.with(
			sampler::InitialiseLaserSamplersSystem,
			"initialise_laser_intensity",
			&["index_cooling_lights"],
		)
		.with(
			gaussian::SampleGaussianBeamIntensitySystem,
			"sample_gaussian_beam_intensity",
			&["initialise_laser_intensity"],
		)
		.with(
			doppler::CalculateDopplerShiftSystem,
			"calculate_doppler_shift",
			&["sample_gaussian_beam_intensity"],
		)
		.with(
			force::CalculateCoolingForcesSystem,
			"calculate_cooling_forces",
			&["calculate_doppler_shift", "sample_gaussian_beam_intensity"],
		)
		.with(
			force::CalculateKickSystem,
			"cal_kick",
			&["sample_gaussian_beam_intensity"],
		)
		.with(repump::RepumpSystem, "repump", &["cal_kick"])
		.with(force::RandomWalkSystem, "random_walk_system", &["cal_kick"])
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<cooling::CoolingLight>();
	world.register::<cooling::CoolingLightIndex>();
	world.register::<sampler::LaserSamplers>();
	world.register::<gaussian::GaussianBeam>();
	world.register::<gaussian::CircularMask>();
}
