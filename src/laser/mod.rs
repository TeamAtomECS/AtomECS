pub mod cooling;
pub mod doppler;
pub mod force;
pub mod gaussian;
pub mod repump;
pub mod sampler;

extern crate specs;
use crate::initiate::NewlyCreated;
use crate::laser::force::NumberScattered;
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
			updater.insert(ent, NumberScattered { value: 0.0 });
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
	builder: &mut DispatcherBuilder<'static, 'static>,
	deps: &[&str],
) -> () {
	builder.add(
		AttachLaserComponentsToNewlyCreatedAtomsSystem,
		"attach_atom_laser_components",
		deps,
	);
	builder.add(
		cooling::AttachIndexToCoolingLightSystem,
		"attach_cooling_index",
		deps,
	);
	builder.add(
		cooling::IndexCoolingLightsSystem,
		"index_cooling_lights",
		&["attach_cooling_index"],
	);
	builder.add(
		sampler::InitialiseLaserSamplersSystem,
		"initialise_laser_intensity",
		&["index_cooling_lights"],
	);
	builder.add(
		gaussian::SampleGaussianBeamIntensitySystem,
		"sample_gaussian_beam_intensity",
		&["initialise_laser_intensity"],
	);
	builder.add(
		doppler::CalculateDopplerShiftSystem,
		"calculate_doppler_shift",
		&["sample_gaussian_beam_intensity"],
	);
	builder.add(
		force::CalculateCoolingForcesSystem,
		"calculate_cooling_forces",
		&["calculate_doppler_shift", "sample_gaussian_beam_intensity"],
	);
	builder.add(
		force::CalculateNumberPhotonsScatteredSystem,
		"cal_kick",
		&["sample_gaussian_beam_intensity"],
	);
	builder.add(repump::RepumpSystem, "repump", &["cal_kick"]);
	builder.add(
		force::ApplyRandomForceSystem,
		"random_walk_system",
		&["cal_kick"],
	);
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<cooling::CoolingLight>();
	world.register::<cooling::CoolingLightIndex>();
	world.register::<sampler::LaserSamplers>();
	world.register::<gaussian::GaussianBeam>();
	world.register::<gaussian::CircularMask>();
}
