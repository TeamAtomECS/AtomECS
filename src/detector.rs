use crate::atom::{Atom, InitialVelocity, Position, Velocity};
use crate::integrator::{Step, Timestep};
extern crate specs;
use specs::{
    Component, DispatcherBuilder, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadExpect,
    ReadStorage, System, VecStorage, World, WriteExpect, WriteStorage,
};

use std::fs::OpenOptions;

use std::error::Error;
extern crate nalgebra;

use nalgebra::Vector3;

/// a Component that clear the csv file that record the informatino about atom detected
pub struct ClearerCSV {
    pub filename: &'static str,
}

impl Component for ClearerCSV {
    type Storage = HashMapStorage<Self>;
}

/// system that clear the csv.file, by default detector.csv
pub struct ClearCSVSystem;

impl<'a> System<'a> for ClearCSVSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, ClearerCSV>);

    fn run(&mut self, (ents, clearer): Self::SystemData) {
        for (entity, clearer) in (&ents, &clearer).join() {
            match clearcsv(clearer.filename) {
                Ok(_) => (),
                Err(why) => panic!("output error{}", why.description()),
            };
            ents.delete(entity).expect("Could not delete entity");
        }
    }
}
/// a resource that record down some of the information about detected atom
pub struct DetectingInfo {
    // I still put it here, because they are quite important parameters to optimize and keeping it is not costly at all
    pub atom_detected: i32,
    pub total_velocity: Vector3<f64>,
}

/// a component that remove the atom that enter its region
/// it has the shape of a cylinder
pub struct Detector {
    /// the radius of the detector
    pub radius: f64,
    /// the thickness/ height of the cylindrical detector
    pub thickness: f64,
    /// direction of the cylindrical detector
    pub direction: Vector3<f64>,
    /// how long the atom needs to be in the detector before it is decided as detected
    /// for an instant detector, this variable should be set to be zero
    pub trigger_time: f64,
    /// the filename of the csv that record the info about captured atoms
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

/// a component indicates that the atom has been detected
pub struct Detected {
    /// time that this atom has been in the detected region.
    pub time: f64,
}

impl Component for Detected {
    type Storage = VecStorage<Self>;
}

/// system used to detecting the atom
pub struct DetectingAtomSystem;

impl<'a> System<'a> for DetectingAtomSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Detector>,
        Entities<'a>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, InitialVelocity>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
        WriteExpect<'a, DetectingInfo>,
        WriteStorage<'a, Detected>,
        Read<'a, LazyUpdate>,
    );
    fn run(
        &mut self,
        (
            pos,
            detector,
            entities,
            atom,
            vel,
            initial_vel,
            step,
            timestep,
            mut detect_info,
            mut detected,
            updater,
        ): Self::SystemData,
    ) {
        let time = step.n as f64 * timestep.delta;
        for (detector_pos, detector) in (&pos, &detector).join() {
            if detector.trigger_time == 0.0 {
                for (atom_pos, _, ent, vel, initial_vel) in
                    (&pos, &atom, &entities, &vel, &initial_vel).join()
                {
                    let rela_pos = atom_pos.pos - detector_pos.pos;
                    if detector.if_detect(&rela_pos) {
                        detect_info.atom_detected = detect_info.atom_detected + 1;
                        detect_info.total_velocity = detect_info.total_velocity + vel.vel;

                        entities.delete(ent).expect("Could not delete entity");
                        let content = vec![
                            vel.vel[0],
                            vel.vel[1],
                            vel.vel[2],
                            initial_vel.vel[0],
                            initial_vel.vel[1],
                            initial_vel.vel[2],
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
            } else {
                for (atom_pos, _, mut detect, ent, vel, initial_vel) in
                    (&pos, &atom, &mut detected, &entities, &vel, &initial_vel).join()
                {
                    let rela_pos = atom_pos.pos - detector_pos.pos;
                    if detector.if_detect(&rela_pos) {
                        detect.time = timestep.delta + detect.time;
                        if detect.time < detector.trigger_time {
                            detect_info.atom_detected = detect_info.atom_detected + 1;
                            detect_info.total_velocity = detect_info.total_velocity + vel.vel;

                            entities.delete(ent).expect("Could not delete entity");
                            let content = vec![
                                vel.vel[0],
                                vel.vel[1],
                                vel.vel[2],
                                initial_vel.vel[0],
                                initial_vel.vel[1],
                                initial_vel.vel[2],
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
                    } else {
                        updater.remove::<Detected>(ent);
                    }
                }
                for (_atom_pos, _atom, (), ent, _vel) in
                    (&pos, &atom, !&detected, &entities, &vel).join()
                {
                    updater.insert(ent, Detected { time: 0.0 });
                }
            }
        }
    }
}
pub fn print_detected_to_file(
    filename: &'static str,
    content: &Vec<f64>,
) -> Result<(), Box<Error>> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    wtr.serialize(&content)?;

    Ok(())
}

pub fn clearcsv(filename: &str) -> Result<(), Box<Error>> {
    let file = OpenOptions::new().write(true).open(filename).unwrap();
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

#[allow(unused_variables)]
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
/// system that print the detected atom info to the output file
pub struct PrintDetectResultSystem;

impl<'a> System<'a> for PrintDetectResultSystem {
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
    let file = OpenOptions::new().write(true).open("output.csv").unwrap();
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

    #[test]
    fn test_detector() {
        let detect = Detector {
            direction: Vector3::new(1., 0., 0.),
            radius: 0.1,
            thickness: 0.1,
            trigger_time: 0.0,
            filename: "detector.csv",
        };
        assert!(detect.if_detect(&Vector3::new(0.04, 0.01, 0.01)));
    }
}
