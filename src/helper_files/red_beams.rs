use crate as lib;
extern crate nalgebra;
use lib::atom::AtomicTransition;
use lib::constant;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use nalgebra::Vector3;
use specs::prelude::*;
use specs::{Builder, World};

pub fn add_red_mot_beams(world: &mut World) {
    let detuning = -0.05; //MHz
    let power = 0.001; //W total power of all Lasers together
    let radius = 3.0e-3 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: -Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();

    // Angled vertical beams
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();

    let detuning = -0.15; //MHz
    let power = 0.001; //W total power of all Lasers together
    let radius = 3.0e-3 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: -Vector3::x(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();

    // Angled vertical beams
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, 1.0, -1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::new(0.0, -1.0, 1.0).normalize(),
            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(
                &(constant::C / AtomicTransition::strontium_red().frequency),
                &radius,
            ),
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
}
