//! Utility for creating simulations with a minimal set of commonly used plugins.

use bevy::{core::TaskPoolThreadAssignmentPolicy, log::LogPlugin, prelude::*};

use crate::{
    destructor::DestroyAtomsPlugin, gravity::GravityPlugin, initiate::InitiatePlugin,
    integrator::IntegrationPlugin, magnetic::MagneticsPlugin,
    output::console_output::console_output, sim_region::SimulationRegionPlugin,
};

/// Used to construct a simulation in AtomECS.
///
/// You can build a simulation in AtomECS by directly adding systems and plugins to your simulation app.
/// This struct provides a convenient way to create a simulation with a minimal set of plugins and resources.
pub struct SimulationBuilder {
    app: App,
}

impl SimulationBuilder {
    pub fn new() -> Self {
        SimulationBuilder { app: App::new() }
    }

    /// Add a [Plugin] to the [SimulationBuilder]
    ///
    /// Plugin dependency should be enforced in individual plugin modules via `app.is_plugin_added`;
    /// panic if the plugins required are not already added.
    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        self.app.add_plugin(plugin);
    }

    /// Finalises the SimulationBuilder and gets the App from it.
    pub fn build(self) -> App {
        self.app
    }
}
impl Default for SimulationBuilder {
    fn default() -> Self {
        let mut builder = Self::new();

        let task_pool_options = TaskPoolOptions {
            // Use 25% of cores for IO, at least 1, no more than 4
            io: TaskPoolThreadAssignmentPolicy {
                min_threads: 0,
                max_threads: 0,
                percent: 0.0,
            },

            // Use 25% of cores for async compute, at least 1, no more than 4
            async_compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 0,
                max_threads: 0,
                percent: 0.0,
            },
            min_total_threads: 1,
            max_total_threads: usize::MAX,
            compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: usize::MAX,
                percent: 100.0,
            },
        };

        builder.app.add_plugin(LogPlugin::default());
        builder.app.add_plugin(TaskPoolPlugin {
            //task_pool_options: TaskPoolOptions::with_num_threads(10),
            task_pool_options,
        });

        builder.app.add_plugin(IntegrationPlugin);
        builder.app.add_plugin(MagneticsPlugin);
        builder.app.add_plugin(SimulationRegionPlugin);
        builder.app.add_plugin(GravityPlugin);
        builder.app.add_plugin(DestroyAtomsPlugin);
        builder.app.add_plugin(InitiatePlugin);
        builder.app.add_system(console_output);
        builder
    }
}

pub const DEFAULT_BEAM_NUMBER: usize = 8;
