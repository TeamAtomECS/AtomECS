use crate::atom::{Atom, Position, Velocity};
use crate::integrator::{Step, Timestep};
extern crate specs;
use specs::{
    Component, Dispatcher, DispatcherBuilder, Entities, HashMapStorage, Join, LazyUpdate, Read,
    ReadExpect, ReadStorage, System, World, WriteExpect, WriteStorage,
};


use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;

use std::error::Error;
extern crate nalgebra;

use crate::destructor::ToBeDestroyed;
use nalgebra::Vector3;

pub struct ClearerCSV {
    pub filename: &'static str,
}

impl Component for ClearerCSV {
    type Storage = HashMapStorage<Self>;
}

pub struct ClearCSVSystem;

impl<'a> System<'a> for ClearCSVSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, ClearerCSV>,
        WriteStorage<'a, ToBeDestroyed>,
    );

    fn run(&mut self, (ents, clearer, mut destroy): Self::SystemData) {
        for (entity, clearer) in (&ents, &clearer).join() {
            match clearcsv(clearer.filename) {
                Ok(_) => (),
                Err(why) => panic!("output error{}", why.description()),
            };
            ents.delete(entity).expect("Could not delete entity");
        }
    }
}

pub struct DetectingInfo {
    // I still put it here, because they are quite important parameters to optimize and keeping it is not costly at all
    pub atom_detected: i32,
    pub total_velocity: Vector3<f64>,
}

pub struct Detector {
    pub radius: f64,
    pub thickness: f64,
    pub direction: Vector3<f64>,
    pub filename: &'static str,
}


impl Detector {
    pub fn if_detect(&self, pos: &Vector3<f64>) -> bool {
        let dir = self.direction.normalize();
        let dis_vertical = dir.dot(&pos);
        let dis_radial = (pos.norm_squared() - dis_vertical.powf(2.0)).powf(0.5);
        (dis_vertical > -0.5 * self.thickness)
            && (dis_vertical < 0.5 * self.thickness)
            && (dis_radial < self.radius)
    }
}

impl Component for Detector {
    type Storage = HashMapStorage<Self>;
}

pub struct DetectingAtomSystem;

impl<'a> System<'a> for DetectingAtomSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Detector>,
        Entities<'a>,
        ReadStorage<'a, Atom>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Velocity>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
        WriteExpect<'a, DetectingInfo>,
    );
    fn run(
        &mut self,
        (pos, detector, entities, atom, lazy, vel, step, timestep, mut detect_info): Self::SystemData,
    ) {
        let time = step.n as f64 * timestep.delta;
        for (detector_pos, detector) in (&pos, &detector).join() {
            for (atom_pos, atom, ent, vel) in (&pos, &atom, &entities, &vel).join() {
                let rela_pos = atom_pos.pos - detector_pos.pos;
                if detector.if_detect(&rela_pos) {
                    println!("atom detected");
                    detect_info.atom_detected = detect_info.atom_detected + 1;
                    detect_info.total_velocity = detect_info.total_velocity + vel.vel;

                    entities.delete(ent).expect("Could not delete entity");
                    let content = vec![
                        vel.vel[0],
                        vel.vel[1],
                        vel.vel[2],
                        atom.initial_velocity[0],
                        atom.initial_velocity[1],
                        atom.initial_velocity[2],
                        time,
                        atom_pos.pos[0],
                        atom_pos.pos[1],
                        atom_pos.pos[2],
                    ];
                    match print_detected_to_file(detector.filename, &content) {
                        Ok(_) => (),
                        Err(why) => panic!("error writing file,{}", why.description()),
                    };
                }
            }
        }
    }
}
pub fn print_detected_to_file(
    filename: &'static str,
    content: &Vec<f64>,
) -> Result<(), Box<Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    wtr.serialize(&content)?;

    Ok(())
}

pub fn clearcsv(filename: &str) -> Result<(), Box<Error>> {
    let file = File::create(filename).expect("Unable to open file");
    let mut file = OpenOptions::new().write(true).open(filename).unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(&[
        "Velocity_Upon_Capture_X",
        "Velocity_Upon_Capture_Y",
        "Velocity_Upon_Capture_Z",
        "Initial_Velocity_X",
        "Initial_Velocity_Y",
        "Initial_Velocity_Z",
        "Time_Captured",
        "Position_Captured_X",
        "Position_Captured_Y",
        "Position_Captured_Z",
    ])?;

    Ok(())
}


pub fn register_components(world: &mut World) {
    world.register::<Detector>();
    world.register::<ClearerCSV>();
}

pub fn add_systems_to_dispatch(
    builder: DispatcherBuilder<'static, 'static>,
    deps: &[&str],
) -> DispatcherBuilder<'static, 'static> {
    builder.with(ClearCSVSystem, "clearcsv", &[]).with(
        DetectingAtomSystem,
        "detect_atom",
        &["clearcsv"],
    )
}

pub struct PrintOptResultSystem;

impl<'a> System<'a> for PrintOptResultSystem {
    type SystemData = (ReadExpect<'a, DetectingInfo>);
    fn run(&mut self, detect_info: Self::SystemData) {
        println!("number detected{}", detect_info.atom_detected);
        match write_file_output(
            detect_info.atom_detected,
            detect_info.total_velocity / (detect_info.atom_detected as f64),
        ) {
            Ok(_) => (),
            Err(why) => panic!("output error{}", why.description()),
        };
    }
}

pub fn write_file_output(number: i32, average_vel: Vector3<f64>) -> Result<(), Box<Error>> {
    let file = File::create("output.csv").expect("Unable to open file");
    let mut file = OpenOptions::new().write(true).open("output.csv").unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    wtr.serialize(&[
        number as f64,
        average_vel[0],
        average_vel[1],
        average_vel[2],
    ])?;

    Ok(())
}
pub mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_imports)]
    extern crate nalgebra;
    extern crate specs;
    use nalgebra::Vector3;

    #[test]
    fn test_detector() {
        let detect = Detector {
            direction: Vector3::new(1., 0., 0.),
            radius: 0.1,
            thickness: 0.1,
            filename: "detector.csv",
        };
        assert!(detect.if_detect(&Vector3::new(0.04, 0.01, 0.01)));
    }
}
