//! Plugin functionality for AtomECS
//! 
//! Allows a simulation to be created in a flexible manner by combining different plugins.

use std::{any::Any};

use specs::prelude::*;

use crate::{magnetic::MagneticsPlugin, atom::{AtomPlugin, ClearForceSystem}, laser::{LaserPlugin}, laser_cooling::{LaserCoolingPlugin, transition::TransitionComponent}, dipole::DipolePlugin, atom_sources::{AtomSourcePlugin, species::AtomCreator}, sim_region::SimulationRegionPlugin, integrator::{VelocityVerletIntegratePositionSystem, INTEGRATE_POSITION_SYSTEM_NAME, INTEGRATE_VELOCITY_SYSTEM_NAME, VelocityVerletIntegrateVelocitySystem, Step}, gravity::GravityPlugin, destructor::DestroyAtomsPlugin, output::console_output::ConsoleOutputSystem};

/// A simulation in AtomECS.
pub struct Simulation {
    pub world: World,
    pub dispatcher: Dispatcher<'static, 'static>
}
impl Simulation {
    pub fn step(&mut self) {
        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
    }
}

/// Used to construct a simulation in AtomECS.
pub struct SimulationBuilder {
    pub world: World,
    pub dispatcher_builder: DispatcherBuilder<'static, 'static>,
    end_frame_systems_added: bool
}
impl SimulationBuilder {
    pub fn new() -> Self {
        let mut dispatcher_builder = DispatcherBuilder::default();

        dispatcher_builder.add(
            VelocityVerletIntegratePositionSystem,
            INTEGRATE_POSITION_SYSTEM_NAME,
            &[],
        );
        dispatcher_builder
            .add(ClearForceSystem, "clear", &[INTEGRATE_POSITION_SYSTEM_NAME]);

        SimulationBuilder {
            world: World::new(),
            dispatcher_builder,
            end_frame_systems_added: false
        }
    }

    /// Add a [Plugin] to the [SimulationBuilder]
    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        plugin.build(self);
    }

    /// Builds a [Simulation] from the [SimulationBuilder].
    pub fn build(mut self) -> Simulation {

        if !self.end_frame_systems_added {
            self.add_end_frame_systems();
        }

        let mut dispatcher = self.dispatcher_builder.build();
        dispatcher.setup(&mut self.world);

        self.world.insert(Step { n: 0 });

        Simulation {
            world: self.world,
            dispatcher: dispatcher
        }
    }

    pub fn add_end_frame_systems(&mut self) {
        self.dispatcher_builder.add(
            VelocityVerletIntegrateVelocitySystem,
            INTEGRATE_VELOCITY_SYSTEM_NAME,
            &[
                "calculate_absorption_forces",
                "calculate_emission_forces",
                "add_gravity",
            ],
        );
        self.dispatcher_builder.add(ConsoleOutputSystem, "", &[INTEGRATE_VELOCITY_SYSTEM_NAME]);
        self.end_frame_systems_added = true;
    }

    /// Configures the simulation to calculate laser cooling forces using the two-level rate equation method.
    /// 
    /// For more information, see [crate::laser_cooling].
    pub fn laser_cooling_via_rate_equation<T>(&mut self)
        where T : TransitionComponent {
            //downside is it hides a lot of the plugin details - better to make that clear?
    }

    pub fn default<T, S>() -> Self 
    where
        T : TransitionComponent,
        S : AtomCreator + 'static
    {
        let mut builder = Self::new();
        builder.add_plugin(AtomPlugin);
        builder.add_plugin(MagneticsPlugin);
        builder.add_plugin(LaserPlugin::<{DEFAULT_BEAM_NUMBER}>);
        builder.add_plugin(LaserCoolingPlugin::<T, {DEFAULT_BEAM_NUMBER}>::default());
        builder.add_plugin(DipolePlugin::<{DEFAULT_BEAM_NUMBER}>);
        builder.add_plugin(AtomSourcePlugin::<S>::default());
        builder.add_plugin(SimulationRegionPlugin);
        builder.add_plugin(GravityPlugin);
        builder.add_plugin(DestroyAtomsPlugin);
        builder
    }
}

pub const DEFAULT_BEAM_NUMBER : usize = 8;

pub trait Plugin : Any + Send + Sync {
    fn build(&self, builder: &mut SimulationBuilder);
}