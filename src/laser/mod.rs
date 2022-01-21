//! Calculation and initialization of laser quantities, eg intensities and indexing.

pub mod frame;
pub mod gaussian;
pub mod index;
pub mod intensity;
pub mod intensity_gradient;
pub mod sampler;

use crate::initiate::NewlyCreated;
use crate::integrator::INTEGRATE_POSITION_SYSTEM_NAME;
use crate::simulation::Plugin;
use specs::prelude::*;

pub const DEFAULT_BEAM_LIMIT: usize = 16;

/// Attaches components used for optical force calculation to newly created atoms.
///
/// They are recognized as newly created if they are associated with
/// the `NewlyCreated` component.
pub struct AttachLaserComponentsToNewlyCreatedAtomsSystem<const N: usize>;

impl<'a, const N: usize> System<'a> for AttachLaserComponentsToNewlyCreatedAtomsSystem<N> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, NewlyCreated>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, newly_created, updater): Self::SystemData) {
        for (ent, _) in (&ent, &newly_created).join() {
            updater.insert(
                ent,
                sampler::CoolingLaserSamplerMasks {
                    contents: [sampler::LaserSamplerMask::default(); N],
                },
            );
            updater.insert(
                ent,
                intensity::LaserIntensitySamplers {
                    contents: [intensity::LaserIntensitySampler::default(); N],
                },
            );
            updater.insert(
                ent,
                intensity_gradient::LaserIntensityGradientSamplers {
                    contents: [intensity_gradient::LaserIntensityGradientSampler::default(); N],
                },
            );
        }
    }
}

/// This plugin provides basic functionality for laser beams, such as calculating laser intensity.
/// 
/// See [crate::laser] for more information.
/// 
/// # Generic Arguments
/// 
/// * `N`: The maximum number of laser beams to configure the simulation for.
pub struct LaserPlugin<const N : usize>;
impl<const N : usize> Plugin for LaserPlugin<N> {
    fn build(&self, builder: &mut crate::simulation::SimulationBuilder) {
        register_components(&mut builder.world);
        add_systems_to_dispatch::<N>(&mut builder.dispatcher_builder, &[]);
    }

    fn deps(&self) -> Vec::<Box<dyn Plugin>> {
        Vec::new()
    }
}

/// Adds the systems required by the module to the dispatcher.
///
/// #Arguments
///
/// `builder`: the dispatch builder to modify
///
/// `deps`: any dependencies that must be completed before the systems run.
fn add_systems_to_dispatch<const N: usize>(
    builder: &mut DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) {
    builder.add(
        AttachLaserComponentsToNewlyCreatedAtomsSystem::<N>,
        "attach_laser_components",
        deps,
    );
    builder.add(index::IndexLasersSystem, "index_lasers", deps);
    builder.add(
        sampler::InitialiseLaserSamplerMasksSystem::<N>,
        "initialise_laser_sampler_masks",
        deps,
    );
    builder.add(
        intensity::InitialiseLaserIntensitySamplersSystem::<N>,
        "initialise_laser_intensity",
        deps,
    );
    builder.add(
        sampler::FillLaserSamplerMasksSystem::<N>,
        "fill_laser_sampler_masks",
        &["index_lasers", "initialise_laser_sampler_masks"],
    );
    builder.add(
        intensity::SampleLaserIntensitySystem::<N>,
        "sample_laser_intensity",
        &[
            "index_lasers",
            "initialise_laser_intensity",
            INTEGRATE_POSITION_SYSTEM_NAME,
        ],
    );
    builder.add(
        intensity_gradient::SampleGaussianLaserIntensityGradientSystem::<N>,
        "sample_intensity_gradient",
        &["index_lasers"],
    );
}

/// Registers resources required by magnetics to the ecs world.
fn register_components(world: &mut World) {
    world.register::<gaussian::GaussianBeam>();
    world.register::<gaussian::CircularMask>();
    world.register::<frame::Frame>();
}
