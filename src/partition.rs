//! Build spatial hashmap of atoms
//!
//! Spatially partition the atoms and construct a hashmap that assigns each atom to a unique cell.
//! This creates a discretised density distribution for use by other systems e.g. tow-body collisions.

extern crate multimap;
use crate::atom::Position;
use hashbrown::HashMap;
use nalgebra::Vector3;
use specs::Entity;
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, VecStorage,
    WriteExpect, WriteStorage,
};

/// Component that marks which boSSx an atom is in for spatial partitioning
pub struct BoxID {
    /// ID of the box
    pub id: i64,
}
impl Component for BoxID {
    type Storage = VecStorage<Self>;
}

/// A partition of space that contains atoms
//#[derive(Clone)]
pub struct PartitionCell {
    pub entities: Vec<Entity>,
    pub density: f64,
    pub volume: f64,
    pub particle_number: i32,
}

impl Default for PartitionCell {
    fn default() -> Self {
        PartitionCell {
            entities: Vec::new(),
            density: 0.0,
            volume: 0.0,
            particle_number: 0,
        }
    }
}

/// Resource for defining spatial partitioning parameters. Space is divided into many small cubes of width box_width, and there are box_number of them
/// along each axis, constituting a large cube of volume (box_number*box_width)^3.
#[derive(Clone, Copy)]
pub struct PartitionParameters {
    /// number of boxes per side in spatial binning
    pub box_number: i64,
    /// width of one box in m
    pub box_width: f64,
    //target density - the number of particles per cell the system will aim to maintain
    pub target_density: f64,
}

impl Default for PartitionParameters {
    fn default() -> Self {
        PartitionParameters {
            box_number: 100,
            box_width: 1e-3,
            target_density: 30.0,
        }
    }
}

pub struct DensityHashmap {
    ///hashmap of density of atoms
    pub hashmap: HashMap<i64, PartitionCell>,
}

impl Default for DensityHashmap {
    fn default() -> Self {
        DensityHashmap {
            hashmap: HashMap::new(),
        }
    }
}

pub struct BuildSpatialPartitionSystem;
impl<'a> System<'a> for BuildSpatialPartitionSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        Entities<'a>,
        ReadStorage<'a, crate::atom::Atom>,
        ReadExpect<'a, PartitionParameters>,
        WriteExpect<'a, DensityHashmap>,
        Read<'a, LazyUpdate>,
        WriteStorage<'a, BoxID>,
    );

    fn run(
        &mut self,
        (
            positions,
            entities,
            atoms,
            partition_params,
            mut hashmap,
            updater,
            mut boxids,
        ): Self::SystemData,
    ) {
        use rayon::prelude::ParallelIterator;
        use specs::ParJoin;
        //make hash table - dividing space up into grid
        // number of boxes per side
        let n_boxes: i64 = partition_params.box_number;
        // Get all atoms which do not have boxIDs
        for (entity, _, _) in (&entities, &atoms, !&boxids).join() {
            updater.insert(entity, BoxID { id: 0 });
        }

        // build list of ids for each atom
        (&positions, &mut boxids)
            .par_join()
            .for_each(|(position, mut boxid)| {
                boxid.id = pos_to_id(position.pos, n_boxes, partition_params.box_width);
            });

        // Count number of particles with each unique box id and insert it into the hashmap
        let mut map: HashMap<i64, PartitionCell> = HashMap::new();
        for (entity, boxid) in (&entities, &boxids).join() {
            if boxid.id == i64::MAX {
                continue;
            } else {
                map.entry(boxid.id).or_default().particle_number += 1;
                map.entry(boxid.id).or_default().entities.push(entity);
            }
        }
        hashmap.hashmap = map;
    }
}

pub struct RescalePartitionCellSystem;
impl<'a> System<'a> for RescalePartitionCellSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, crate::atom::Atom>,
        ReadExpect<'a, DensityHashmap>,
        WriteExpect<'a, PartitionParameters>,
    );

    fn run(&mut self, (positions, atoms, hashmap, mut partition_params): Self::SystemData) {
        // calculate box number and box width based on atom density
        // for low collision fluctuations we want >30 particles per cell
        // we also want cells to be smaller than typical density variation
        //
        // take the existing hashmap
        // calculate average number of particles per cell
        // we want this to be (~30?)
        // so then rescale the cell size by whatever number is required to make
        // the average n = 30 (or whatever the target_density is set to)

        // Rescale has to run before Build, or else the density map that Build writes
        // won't match the partition parameters after we rescale them
        // But in the first step no hashmap exists yet so if the map is empty we pass

        let map = &hashmap.hashmap;
        let cells: Vec<&PartitionCell> = map.values().collect();

        if cells.is_empty() {
        } else {
            //// get size of cloud
            let mut xs: Vec<f64> = Vec::new();
            let mut ys: Vec<f64> = Vec::new();
            let mut zs: Vec<f64> = Vec::new();

            for (position, _atom) in (&positions, &atoms).join() {
                xs.push(position.pos[0]);
                ys.push(position.pos[1]);
                zs.push(position.pos[2]);
            }
            let xrange = get_max(&xs) - get_min(&xs);
            let yrange = get_max(&ys) - get_min(&ys);
            let zrange = get_max(&zs) - get_min(&zs);

            let max_range = get_max(&vec![xrange, yrange, zrange]);
            let min_range = get_min(&vec![xrange, yrange, zrange]);

            //// Get average particles per cell

            let mut total: i32 = 0;
            for cell in &cells {
                total += cell.particle_number;
            }
            let average_particles_per_cell = total as f64 / cells.len() as f64;
            // maximum allowable box width is 1/10th of cloud
            // This is to ensure partition cells are smaller than large density variations in cloud
            let max_box_width = min_range / 10.0;

            // make volume larger by target_density/average_particles, so box_width scales by cube root of this
            let scale_factor =
                (partition_params.target_density / average_particles_per_cell).powf(1.0 / 3.0);
            // Set box width to the rescaled box size, or the maximum box size, whichever is smaller
            partition_params.box_width =
                (partition_params.box_width * scale_factor).min(max_box_width);

            let box_number = max_range / partition_params.box_width;
            partition_params.box_number = box_number.ceil() as i64;
        }
    }
}

fn get_min(x: &Vec<f64>) -> f64 {
    x.iter()
        .cloned()
        .min_by(|a, b| a.partial_cmp(b).expect("Tried to compare a NaN"))
        .unwrap()
}
fn get_max(x: &Vec<f64>) -> f64 {
    x.iter()
        .cloned()
        .max_by(|a, b| a.partial_cmp(b).expect("Tried to compare a NaN"))
        .unwrap()
}

fn pos_to_id(pos: Vector3<f64>, n: i64, width: f64) -> i64 {
    //Assume that atoms that leave the grid are too sparse to collide, so disregard them
    //We'll assign them the max value of i64, and then check for this value when we do a collision and ignore them
    let bound = (n as f64) / 2.0 * width;

    let id: i64;
    if pos[0].abs() > bound {
        id = i64::MAX;
    } else if pos[1].abs() > bound {
        id = i64::MAX;
    } else if pos[2].abs() > bound {
        id = i64::MAX;
    } else {
        let xp: i64;
        let yp: i64;
        let zp: i64;

        // even number of boxes, vertex of a box is on origin
        // odd number of boxes, centre of a box is on the origin
        // grid cells run from [0, width), i.e include lower bound but exclude upper

        xp = (pos[0] / width + 0.5 * (n as f64)).floor() as i64;
        yp = (pos[1] / width + 0.5 * (n as f64)).floor() as i64;
        zp = (pos[2] / width + 0.5 * (n as f64)).floor() as i64;
        //convert position to box id
        id = xp + n * yp + n.pow(2) * zp;
    }
    id
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::atom::{Atom, Force, Mass, Position, Velocity};
    #[allow(unused_imports)]
    use crate::ecs;
    #[allow(unused_imports)]
    use crate::ecs::AtomecsDispatcherBuilder;
    #[allow(unused_imports)]
    use crate::initiate::NewlyCreated;
    #[allow(unused_imports)]
    use crate::integrator::{
        Step, Timestep, VelocityVerletIntegratePositionSystem,
        VelocityVerletIntegrateVelocitySystem,
    };

    #[allow(unused_imports)]
    use nalgebra::Vector3;
    #[allow(unused_imports)]
    use specs::prelude::*;
    extern crate specs;

    #[test]
    fn test_pos_to_id() {
        let n: i64 = 10;
        let width: f64 = 2.0;

        let pos1 = Vector3::new(0.0, 0.0, 0.0);
        let pos2 = Vector3::new(1.0, 0.0, 0.0);
        let pos3 = Vector3::new(2.0, 0.0, 0.0);
        let pos4 = Vector3::new(9.9, 0.0, 0.0);
        let pos5 = Vector3::new(-9.9, 0.0, 0.0);
        let pos6 = Vector3::new(10.1, 0.0, 0.0);
        let pos7 = Vector3::new(-9.9, -9.9, -9.9);

        let id1 = pos_to_id(pos1, n, width);
        let id2 = pos_to_id(pos2, n, width);
        let id3 = pos_to_id(pos3, n, width);
        let id4 = pos_to_id(pos4, n, width);
        let id5 = pos_to_id(pos5, n, width);
        let id6 = pos_to_id(pos6, n, width);
        let id7 = pos_to_id(pos7, n, width);

        assert_eq!(id1, 555);
        assert_eq!(id2, 555);
        assert_eq!(id3, 556);
        assert_eq!(id4, 559);
        assert_eq!(id5, 550);
        assert_eq!(id6, i64::MAX);
        assert_eq!(id7, 0);
    }

    #[test]

    fn test_build_spatial_partition() {
        let mut test_world = World::new();

        let mut system = BuildSpatialPartitionSystem;
        test_world.register::<Position>();
        test_world.register::<Atom>();
        test_world.register::<BoxID>();
        let n_boxes = 10;
        let width = 2.0;

        let atom1 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(1.0, 0.0, 0.0),
            })
            .with(Atom)
            .build();
        let atom2 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(1.0, 1.0, 0.0),
            })
            .with(Atom)
            .build();
        let atom3 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(-1.0, 1.0, 1.0),
            })
            .with(Atom)
            .build();
        let atom4 = test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 3.0, 0.0),
            })
            .with(Atom)
            .build();

        test_world.insert(Timestep { delta: 1.0 });
        test_world.insert(Step { n: 0 });
        test_world.insert(DensityHashmap::default());

        test_world.insert(PartitionParameters {
            box_number: n_boxes,
            box_width: width,
            target_density: 1.0,
        });

        for _i in 0..2 {
            system.run_now(&test_world);
            test_world.maintain();
        }

        let boxid_storage = test_world.read_storage::<BoxID>();

        let boxid1 = boxid_storage.get(atom1).expect("entity not found").id;
        let boxid2 = boxid_storage.get(atom2).expect("entity not found").id;
        let boxid3 = boxid_storage.get(atom3).expect("entity not found").id;
        let boxid4 = boxid_storage.get(atom4).expect("entity not found").id;

        assert!(boxid1 == 555);
        assert!(boxid2 == 555);
        assert!(boxid3 == 554);
        assert!(boxid4 == 565);
    }

    #[test]
    fn test_rescale_spatial_partition() {
        let mut test_world = World::new();

        let mut build_system = BuildSpatialPartitionSystem;
        let mut rescale_system = RescalePartitionCellSystem;
        test_world.register::<Position>();
        test_world.register::<Atom>();
        test_world.register::<BoxID>();
        let n_boxes = 10;
        let width = 2.0;

        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(1.9, 1.9, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();
        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(-1.9, 1.9, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();
        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(1.9, -1.9, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();
        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(-1.9, -1.9, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();

        test_world.insert(Timestep { delta: 1.0 });
        test_world.insert(Step { n: 0 });
        test_world.insert(DensityHashmap::default());

        test_world.insert(PartitionParameters {
            box_number: n_boxes,
            box_width: width,
            target_density: 1.0,
        });

        build_system.run_now(&test_world);
        test_world.maintain();

        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.5, 0.5, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();
        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(-0.5, 0.5, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();
        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.5, -0.5, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();
        test_world
            .create_entity()
            .with(Position {
                pos: Vector3::new(-0.5, -0.5, 0.5),
            })
            .with(Atom)
            .with(BoxID { id: 0 })
            .build();

        build_system.run_now(&test_world);
        rescale_system.run_now(&test_world);
        test_world.maintain();

        let read_params = test_world.read_resource::<PartitionParameters>();

        println!("{}", &read_params.box_width);
        println!("{}", &read_params.box_number);
        assert_eq!(read_params.box_number, 3);
        assert_approx_eq::assert_approx_eq!(read_params.box_width, 2.0 / (2.0_f64).powf(1.0 / 3.0))
    }
}
