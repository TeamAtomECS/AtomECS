extern crate rayon;
extern crate specs;
use crate::atom::{Atom, Position, Velocity};
use crate::integrator::Step;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage, Join, ReadExpect, ReadStorage, System, WriteStorage};
use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct XYZWriteHelper {
    pub overwrite: bool,
    pub initialized: bool,
    pub write_every: u64,
    pub scale_factor: f64,
    pub discard_place: Vector3<f64>,
    pub name: String,
}

impl Default for XYZWriteHelper {
    fn default() -> Self {
        XYZWriteHelper {
            overwrite: true,
            initialized: false,
            write_every: 100,
            scale_factor: 20000.,
            discard_place: Vector3::new(0., 0., 0.),
            name: format!("{}", "pos_xyz"),
        }
    }
}

impl Component for XYZWriteHelper {
    type Storage = HashMapStorage<Self>;
}

pub struct WriteToXYZFileSystem;
impl<'a> System<'a> for WriteToXYZFileSystem {
    type SystemData = (
        ReadExpect<'a, Step>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, XYZWriteHelper>,
    );

    fn run(&mut self, (step_number, atom, velocity, position, mut xyz_helper): Self::SystemData) {
        for helper in (&mut xyz_helper).join() {
            if (step_number.n % helper.write_every == 0 && step_number.n != 0) || step_number.n == 2
            {
                let mut data_string = String::new();
                if helper.initialized != true {
                    if helper.overwrite == true {
                        use std::fs;
                        if let Err(e) = fs::remove_file(format!("{}.xyz", helper.name).as_str()) {
                            eprintln!("Couldn't delete old file: {}", e);
                        }
                    }
                    data_string.push_str(format!("{}\n\n", (&atom).join().count()).as_str());
                    for _ in 0..(&atom).join().count() {
                        data_string.push_str(
                            format!(
                                "H\t{}\t{}\t{}\n",
                                &(helper.discard_place[0]).to_string(),
                                &(helper.discard_place[1]).to_string(),
                                &(helper.discard_place[2]).to_string()
                            )
                            .as_str(),
                        );
                    }
                    helper.initialized = true;
                }
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(format!("{}.xyz", helper.name).as_str())
                    .unwrap();
                let atom_number = (&atom).join().count();
                data_string.push_str(format!("{}\n\n", atom_number).as_str());
                for (_, _, pos) in (&atom, &velocity, &position).join() {
                    data_string.push_str(
                        format!(
                            "H\t{}\t{}\t{}\n",
                            &(helper.scale_factor * pos.pos[0]).to_string(),
                            &(helper.scale_factor * pos.pos[1]).to_string(),
                            &(helper.scale_factor * pos.pos[2]).to_string()
                        )
                        .as_str(),
                    );
                }
                if let Err(e) = write!(file, "{}", data_string) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::atom::Force;
    use crate::atom::Mass;
    use crate::integrator::EulerIntegrationSystem;
    use crate::integrator::Timestep;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    #[test]
    fn test_write_to_xyz_file_system() {
        let mut test_world = World::new();

        test_world.register::<Position>();
        test_world.register::<Velocity>();
        test_world.register::<Atom>();
        test_world.register::<XYZWriteHelper>();
        test_world.register::<Force>();
        test_world.register::<Mass>();
        test_world.add_resource(Step { n: 0 });
        test_world.add_resource(Timestep { delta: 1.0e-6 });

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(1.0, 0.0, 0.0),
            })
            .with(Velocity {
                vel: Vector3::new(1.0, 0.0, 0.0),
            })
            .with(Atom {})
            .with(Mass { value: 1. })
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .build();

        let xyz_helper = test_world
            .create_entity()
            .with(XYZWriteHelper {
                overwrite: true,
                initialized: false,
                write_every: 100,
                scale_factor: 20000.,
                discard_place: Vector3::new(0.0, 0.0, 0.0),
                name: format!("{}", "test_xyz"),
            })
            .build();

        let mut write_system = WriteToXYZFileSystem;
        let mut int_system = EulerIntegrationSystem;
        for _ in 0..1000 {
            int_system.run_now(&test_world.res);
            write_system.run_now(&test_world.res);
            test_world.maintain();
        }
        let sampler_storage = test_world.read_storage::<Position>();
        let writer_storage = test_world.read_storage::<XYZWriteHelper>();

        use std::fs;
        if let Err(e) = fs::remove_file(
            format!(
                "{}.xyz",
                writer_storage
                    .get(xyz_helper)
                    .expect("entity not found")
                    .name
            )
            .as_str(),
        ) {
            eprintln!("Couldn't delete old file: {}", e);
        }
        assert_approx_eq!(
            sampler_storage.get(atom1).expect("entity not found").pos[0],
            1.0 + 1.0e-4,
            1e-3_f64
        );
    }
}
