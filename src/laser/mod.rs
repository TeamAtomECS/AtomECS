pub mod cooling;
pub mod doppler;
pub mod force;
pub mod gaussian;
pub mod intensity;
pub mod photons_scattered;
pub mod rate;
pub mod repump;
pub mod sampler;
pub mod twolevel;

extern crate specs;
use crate::initiate::NewlyCreated;
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
				sampler::LightWavePropertiesSamplers {
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
			updater.insert(
				ent,
				rate::RateCoefficients {
					contents: Vec::new(),
				},
			);
			updater.insert(ent, twolevel::TwoLevelPopulation::default());
			updater.insert(ent, photons_scattered::TotalPhotonsScattered::default());
			updater.insert(
				ent,
				photons_scattered::ExpectedPhotonsScatteredVector {
					contents: Vec::new(),
				},
			);
			updater.insert(
				ent,
				photons_scattered::ActualPhotonsScatteredVector {
					contents: Vec::new(),
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
			sampler::InitialiseLightWavePropertiesSamplersSystem, // will become unneccessary/changed
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
			rate::InitialiseRateCoefficientsSystem,
			"initialise_rate_coefficient",
			&["initialise_laser_detuning"],
		)
		.with(
			photons_scattered::InitialiseExpectedPhotonsScatteredVectorSystem,
			"initialise_expected_photons",
			&["initialise_rate_coefficient"],
		)
		.with(
			photons_scattered::InitialiseActualPhotonsScatteredVectorSystem,
			"initialise_actual_photons",
			&["initialise_expected_photons"],
		)
		.with(
			intensity::SampleLaserIntensitySystem,
			"sample_laser_intensity",
			&["initialise_actual_photons"],
		)
		.with(
			doppler::CalculateDopplerShiftSystem,
			"calculate_doppler_shift",
			&["sample_laser_intensity"],
		)
		.with(
			sampler::CalculateLaserDetuningSystem,
			"calculate_laser_detuning",
			&["calculate_doppler_shift"],
		)
		.with(
			rate::CalculateRateCoefficientsSystem,
			"calculate_rate_coefficients",
			&["calculate_laser_detuning"],
		)
		.with(
			twolevel::CalculateTwoLevelPopulationSystem,
			"calculate_twolevel",
			&["calculate_rate_coefficients"],
		)
		.with(
			photons_scattered::CalculateMeanTotalPhotonsScatteredSystem,
			"calculate_total_photons",
			&["calculate_twolevel"],
		)
		.with(
			photons_scattered::CalculateExpectedPhotonsScatteredSystem,
			"calculate_expected_photons",
			&["calculate_total_photons"],
		)
		.with(
			photons_scattered::CalculateActualPhotonsScatteredSystem,
			"calculate_actual_photons",
			&["calculate_expected_photons"],
		)
		.with(
			force::CalculateAbsorptionForcesSystem,
			"calculate_absorption_forces",
			&["calculate_actual_photons", "sample_gaussian_beam_intensity"],
		)
		.with(repump::RepumpSystem, "repump", &["cal_kick"])
		.with(
			force::ApplyEmissionForceSystem,
			"random_walk_system",
			&["cal_kick"],
		)
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<cooling::CoolingLight>();
	world.register::<cooling::CoolingLightIndex>();
	world.register::<sampler::LightWavePropertiesSamplers>();
	world.register::<gaussian::GaussianBeam>();
	world.register::<gaussian::CircularMask>();
}
