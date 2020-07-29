//! Loading a Rb pyramid MOT from vapor
//!
//! One of the beams has a circular mask, which allows atoms to escape on one side.

extern crate magneto_optical_trap as lib;
extern crate nalgebra;
use lib::atom::{AtomInfo, Position, Velocity};
use lib::atom_sources::emit::{AtomNumberToEmit, EmitOnce};
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::atom_sources::surface::SurfaceSource;
use lib::atom_sources::VelocityCap;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::force::RandomScatteringForceOption;
use lib::laser::gaussian::{CircularMask, GaussianBeam};
use lib::magnetic::grid::PrecalculatedMagneticFieldGrid;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use lib::shapes::Cylinder;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
use serde::Deserialize;
use specs::{Builder, RunNow, World};
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

#[derive(Deserialize)]
pub struct SimulationParameters {
    pub cooling_detuning: f64,
    pub cooling_power: f64,
    pub cooling_radius: f64,

    pub beam_x_p_pol: f64,
    pub beam_x_m_pol: f64,
    pub beam_y_p_pol: f64,
    pub beam_y_m_pol: f64,
    pub beam_z_p_pol: f64,
    pub beam_z_m_pol: f64,

    pub beam_x_p_dir: Vector3<f64>,
    pub beam_x_m_dir: Vector3<f64>,
    pub beam_y_p_dir: Vector3<f64>,
    pub beam_y_m_dir: Vector3<f64>,
    pub beam_z_p_dir: Vector3<f64>,
    pub beam_z_m_dir: Vector3<f64>,

    pub beam_z_mask_radius: f64,

    pub trap_x: f64,
    pub trap_y: f64,
    pub trap_z: f64,

    pub quadrupole_gradient: f64,
    pub quadrupole_x: f64,
    pub quadrupole_y: f64,
    pub quadrupole_z: f64,

    pub chamber_length: f64,
    pub chamber_radius: f64,
    pub atom_number: i32,
    pub velocity_cap: f64,
    pub n_steps: i32,

    pub use_grid_field: bool,
}

fn main() {
    let now = Instant::now();

    let json_str = read_to_string("input.json").expect("Could not open file");
    println!("Loaded json string: {}", json_str);
    let parameters: SimulationParameters = serde_json::from_str(&json_str).unwrap();

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // Create magnetic field.
    if parameters.use_grid_field {
        let f = File::open("field.json").expect("Could not open file.");
        let reader = BufReader::new(f);
        let grid: PrecalculatedMagneticFieldGrid = serde_json::from_reader(reader)
            .expect("Could not load magnetic field grid from json file.");
        world.create_entity().with(grid).build();
    } else {
        let field_centre = Vector3::new(
            parameters.quadrupole_x,
            parameters.quadrupole_y,
            parameters.quadrupole_z,
        );
        world
            .create_entity()
            .with(QuadrupoleField3D::gauss_per_cm(
                parameters.quadrupole_gradient,
                Vector3::z(),
            ))
            .with(Position { pos: field_centre })
            .build();
    }

    // Create cooling lasers.
    let detuning = parameters.cooling_detuning;
    let power = parameters.cooling_power;
    let radius = parameters.cooling_radius / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(parameters.trap_x, parameters.trap_y, parameters.trap_z);

    // Horizontal beams along z
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: parameters.beam_z_p_dir,
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            parameters.beam_z_p_pol,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: parameters.beam_z_m_dir,
        })
        .with(CircularMask {
            radius: parameters.beam_z_mask_radius,
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            parameters.beam_z_m_pol,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: parameters.beam_x_p_dir,
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            parameters.beam_x_p_pol,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: parameters.beam_x_m_dir,
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            parameters.beam_x_m_pol,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: parameters.beam_y_p_dir,
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            parameters.beam_y_p_pol,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: beam_centre.clone(),
            e_radius: radius,
            power: power,
            direction: parameters.beam_y_m_dir,
        })
        .with(CoolingLight::for_species(
            AtomInfo::rubidium(),
            detuning,
            parameters.beam_y_m_pol,
        ))
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });
    world.add_resource(RandomScatteringForceOption);

    // The simulation bounds consists of a vertical cylinder. It also emits atoms from the surface.
    let number_to_emit = parameters.atom_number;
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cylinder::new(
            parameters.chamber_radius,
            parameters.chamber_length,
            Vector3::new(0.0, 0.0, 1.0),
        ))
        .with(SurfaceSource { temperature: 300.0 })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 87.0,
            ratio: 1.0,
        }]))
        .with(AtomInfo::rubidium())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(EmitOnce {})
        .build();

    // Also use a velocity cap so that fast atoms are not even simulated.
    world.add_resource(VelocityCap {
        value: parameters.velocity_cap,
    });
    let mut output_vel_sys = file::new::<Velocity, Text>("vel.txt".to_string(), 100);

    let mut output_pos_sys = file::new::<Position, Text>("pos.txt".to_string(), 100);

    // Run the simulation for a number of steps.
    for _i in 0..parameters.n_steps {
        output_vel_sys.run_now(&world.res);
        output_pos_sys.run_now(&world.res);
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
