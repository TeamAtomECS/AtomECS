//! Calculation and initialization of optical forces and quantities exerted on the atoms

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
use crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME;
use specs::{DispatcherBuilder, Entities, Join, LazyUpdate, Read, ReadStorage, System, World};

pub const COOLING_BEAM_LIMIT: usize = 16;

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
					contents: [sampler::LaserSamplerMask::default(); COOLING_BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				doppler::DopplerShiftSamplers {
					contents: [doppler::DopplerShiftSampler::default(); COOLING_BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				intensity::LaserIntensitySamplers {
					contents: [intensity::LaserIntensitySampler::default(); COOLING_BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				sampler::LaserDetuningSamplers {
					contents: [sampler::LaserDetuningSampler::default(); COOLING_BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				rate::RateCoefficients {
					contents: [rate::RateCoefficient::default(); COOLING_BEAM_LIMIT],
				},
			);
			updater.insert(ent, twolevel::TwoLevelPopulation::default());
			updater.insert(ent, photons_scattered::TotalPhotonsScattered::default());
			updater.insert(
				ent,
				photons_scattered::ExpectedPhotonsScatteredVector {
					contents: [photons_scattered::ExpectedPhotonsScattered::default();
						COOLING_BEAM_LIMIT],
				},
			);
			updater.insert(
				ent,
				photons_scattered::ActualPhotonsScatteredVector {
					contents: [photons_scattered::ActualPhotonsScattered::default();
						COOLING_BEAM_LIMIT],
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
		deps,
	);
	builder.add(
		sampler::InitialiseLaserSamplerMasksSystem,
		"initialise_laser_sampler_masks",
		deps,
	);
	builder.add(
		sampler::FillLaserSamplerMasksSystem,
		"fill_laser_sampler_masks",
		&["index_cooling_lights", "initialise_laser_sampler_masks"],
	);
	builder.add(
		intensity::SampleLaserIntensitySystem,
		"sample_laser_intensity",
		&["index_cooling_lights", INTEGRATE_POSITION_SYSTEM_NAME],
	);
	builder.add(
		doppler::CalculateDopplerShiftSystem,
		"calculate_doppler_shift",
		&["index_cooling_lights"],
	);
	builder.add(
		sampler::CalculateLaserDetuningSystem,
		"calculate_laser_detuning",
		&[
			"calculate_doppler_shift",
			"zeeman_shift",
			"index_cooling_lights",
		],
	);
	builder.add(
		rate::CalculateRateCoefficientsSystem,
		"calculate_rate_coefficients",
		&["calculate_laser_detuning"],
	);
	builder.add(
		twolevel::CalculateTwoLevelPopulationSystem,
		"calculate_twolevel",
		&["calculate_rate_coefficients", "fill_laser_sampler_masks"],
	);
	builder.add(
		photons_scattered::CalculateMeanTotalPhotonsScatteredSystem,
		"calculate_total_photons",
		&["calculate_twolevel"],
	);
	builder.add(
		photons_scattered::CalculateExpectedPhotonsScatteredSystem,
		"calculate_expected_photons",
		&["calculate_total_photons", "fill_laser_sampler_masks"],
	);
	builder.add(
		photons_scattered::CalculateActualPhotonsScatteredSystem,
		"calculate_actual_photons",
		&["calculate_expected_photons"],
	);
	builder.add(
		force::CalculateAbsorptionForcesSystem,
		"calculate_absorption_forces",
		&["calculate_actual_photons", INTEGRATE_POSITION_SYSTEM_NAME],
	);
	builder.add(
		repump::RepumpSystem,
		"repump",
		&["calculate_absorption_forces"],
	);
	builder.add(
		force::ApplyEmissionForceSystem,
		"calculate_emission_forces",
		&[
			"calculate_absorption_forces",
			INTEGRATE_POSITION_SYSTEM_NAME,
		],
	);
}

/// Registers resources required by magnetics to the ecs world.
pub fn register_components(world: &mut World) {
	world.register::<cooling::CoolingLight>();
	world.register::<cooling::CoolingLightIndex>();
	world.register::<gaussian::GaussianBeam>();
	world.register::<gaussian::CircularMask>();
	world.register::<gaussian::GaussianRayleighRange>();
	world.register::<gaussian::GaussianReferenceFrame>();
}
