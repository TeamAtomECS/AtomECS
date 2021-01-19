pub mod cooling;
pub mod doppler;
pub mod force;
pub mod gaussian;
pub mod intensity;
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
			updater.insert(
				ent,
				doppler::DopplerShiftSamplers {
					contents: Vec::new(),
				},
			);
			updater.insert(
				ent,
				intensity::LaserIntensitySamplers {
					contents: Vec::new(),
				},
			);
			updater.insert(
				ent,
				sampler::LaserDetuningSamplers {
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
			sampler::InitialiseLaserSamplersSystem, // will become unneccessary/changed
			"initialise_laser_sampler",
			&["index_cooling_lights"],
		)
		.with(
			intensity::InitialiseLaserIntensitySamplersSystem,
			"initialise_laser_intensity",
			&["initialise_laser_sampler"],
		)
		.with(
			doppler::InitialiseDopplerShiftSamplersSystem,
			"initialise_doppler_shift",
			&["initialise_laser_intensity"],
		)
		.with(
			sampler::InitialiseLaserDetuningSamplersSystem,
			"initialise_laser_detuning",
			&["initialise_doppler_shift"],
		)
		.with(
			intensity::SampleLaserIntensitySystem,
			"sample_laser_intensity",
			&["initialise_laser_intensity"],
		)
		.with(
			gaussian::SampleGaussianBeamIntensitySystem, // delete later, currently only doing the polarization and wave-vector, intensity redundant
			"sample_gaussian_beam_intensity",
			&["sample_laser_intensity"],
		)
		.with(
			doppler::CalculateDopplerShiftSystem,
			"calculate_doppler_shift",
			&["sample_gaussian_beam_intensity"],
		)
		.with(
			sampler::CalculateLaserDetuningSystem,
			"calculate_laser_detuning",
			&["calculate_doppler_shift"],
		)
		.with(
			force::CalculateCoolingForcesSystem, //to be superseeded
			"calculate_cooling_forces",
			&["calculate_laser_detuning", "sample_gaussian_beam_intensity"],
		)
		.with(
			force::CalculateNumberPhotonsScatteredSystem,
			"cal_kick",
			&["sample_gaussian_beam_intensity"],
		)
		.with(repump::RepumpSystem, "repump", &["cal_kick"])
		.with(
			force::ApplyRandomForceSystem,
			"random_walk_system",
			&["cal_kick"],
		)
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<cooling::CoolingLight>();
	world.register::<cooling::CoolingLightIndex>();
	world.register::<sampler::LaserSamplers>();
	world.register::<gaussian::GaussianBeam>();
	world.register::<gaussian::CircularMask>();
}
