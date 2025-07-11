//! Utility for creating simulations with a minimal set of commonly used plugins.

use bevy::{app::TaskPoolThreadAssignmentPolicy, log::LogPlugin, prelude::*};

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
    pub fn add_plugins(&mut self, plugin: impl Plugin) {
        self.app.add_plugins(plugin);
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
                on_thread_spawn: None,
                on_thread_destroy: None,
            },

            // Use 25% of cores for async compute, at least 1, no more than 4
            async_compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 0,
                max_threads: 0,
                percent: 0.0,
                on_thread_spawn: None,
                on_thread_destroy: None,
            },
            min_total_threads: 1,
            max_total_threads: usize::MAX,
            compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: usize::MAX,
                percent: 100.0,
                on_thread_spawn: None,
                on_thread_destroy: None,
            },
        };

        builder.app.add_plugins(LogPlugin::default());
        builder.app.add_plugins(TaskPoolPlugin {
            //task_pool_options: TaskPoolOptions::with_num_threads(10),
            task_pool_options,
        });

        builder.app.add_plugins(IntegrationPlugin);
        builder.app.add_plugins(MagneticsPlugin);
        builder.app.add_plugins(SimulationRegionPlugin);
        builder.app.add_plugins(GravityPlugin);
        builder.app.add_plugins(DestroyAtomsPlugin);
        builder.app.add_plugins(InitiatePlugin);
        builder.app.add_systems(Update, console_output);
        builder
    }
}

pub const DEFAULT_BEAM_NUMBER: usize = 8;
