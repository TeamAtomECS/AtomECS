//! Implements s-wave scattering of atoms
//! We use here a standard Direct Simulation Monte Carlo method of simulating collisions. For much greater detail on these alogrithms see e.g.
//! Molecular Gas Dynamics and the Direct Simulation of Gas Flows 1998 by G.A. Bird.
//! We here divide the space into a grid of collision cells within which collisiosn can occur. Based on simple kinetic theory we predict how many collisions
//! should occur within each box based on density and average velocity, and randomly select this many pairs of atoms to collide.
//!
//! # Limitations
//! We assume the atoms within a cell have an approximately thermal distribution in order to relate average velocity to average relative velocity.
//! For cases where this approximation is poor, the collision rate may be wrong.
//! We assume a single species of atom, with a constant (not velocity dependent) collisional cross-section.
//!
//!
//!

extern crate multimap;
use crate::atom::Velocity;
use crate::constant::{PI, SQRT2};
use crate::integrator::Timestep;
use crate::partition::{PartitionCell, PartitionParameters, VelocityHashmap};
use nalgebra::Vector3;
use rand::Rng;
use specs::{Component, Read, ReadExpect, ReadStorage, System, VecStorage, WriteExpect};

/// A resource that indicates that the simulation should apply scattering
pub struct ApplyCollisionsOption;

impl PartitionCell {
    /// Perform collisions within a box.
    fn do_collisions(
        &mut self,
        partition_params: PartitionParameters,
        collision_params: CollisionParameters,
        dt: f64,
    ) {
        let mut rng = rand::thread_rng();
        self.particle_number = self.velocities.len() as i32;
        self.atom_number = self.particle_number as f64 * collision_params.macroparticle;
        // Only one atom or less in box -> no collisions.
        if self.particle_number <= 1 {
            return;
        }

        ///// n*sigma*v total collisions
        // vbar is the average _speed_, not the average _velocity_.
        let mut vsum = 0.0;
        for i in 0..self.velocities.len() {
            vsum = vsum + self.velocities[i].vel.norm();
        }
        let vbar = vsum / self.velocities.len() as f64;

        println!("vbar: {}", vbar);
        // number of collisions is N*n*sigma*v*dt, where n is atom density and N is atom number
        // probability of one particle colliding is n*sigma*vrel*dt where n is the atom density, sigma cross section and vrel the average relative velocity
        // vrel = SQRT(2)*vbar, and since we assume these are identical particles we must divide by two since otherwise we count each collision twice
        // so total number of collisions is N_particles * probability = N_p*n*sigma*vbar*dt/SQRT(2)
        let density = self.atom_number / partition_params.box_width.powi(3);
        self.expected_collision_number = self.particle_number as f64
            * density
            * collision_params.sigma
            * vbar
            * dt
            * (1.0 / SQRT2);

        println!(
            "expected collision number: {}",
            self.expected_collision_number
        );
        let mut num_collisions_left: f64 = self.expected_collision_number;

        if num_collisions_left > collision_params.collision_limit {
            panic!("Number of collisions in a box in a single frame exceeds limit. Number of collisions={}, limit={}, particles={}.", num_collisions_left, collision_params.collision_limit, self.particle_number);
        }

        while num_collisions_left > 0.0 {
            let collide = if num_collisions_left > 1.0 {
                true
            } else {
                rng.gen::<f64>() < num_collisions_left
            };

            if collide {
                let idx1 = rng.gen_range(0..self.velocities.len());
                let mut idx2 = idx1;
                while idx2 == idx1 {
                    idx2 = rng.gen_range(0..self.velocities.len())
                }

                let v1 = self.velocities[idx1].vel;
                let v2 = self.velocities[idx2].vel;
                let (v1new, v2new) = do_collision(v1, v2);
                self.velocities[idx1].vel = v1new;
                self.velocities[idx2].vel = v2new;
                self.collision_number += 1;
                println!("v1:{}, v1new: {}", v1, v1new);
            }

            num_collisions_left -= 1.0;
        }
        /////
    }
}

/// Resource for defining collision relevant paramaters like macroparticle number, box width and number of boxes
#[derive(Copy, Clone)]
pub struct CollisionParameters {
    /// number of real particles one simulation particle represents for collisions
    pub macroparticle: f64,
    // collisional cross section of atoms (assuming only one species)
    pub sigma: f64,
    /// Limit on number of collisions per box each frame. If the number of collisions to calculate exceeds this, the simulation will panic.
    pub collision_limit: f64,
}

/// store stats about collisions
#[derive(Clone)]
pub struct CollisionsTracker {
    /// number of collisions in each box
    pub num_collisions: Vec<i32>,
    /// number of simulated particles in each box
    pub num_particles: Vec<i32>,
    /// number of simulated atoms in each box
    pub num_atoms: Vec<f64>,
}

/// Performs collisions within the atom cloud using a spatially partitioned Monte-Carlo approach.
pub struct ApplyCollisionsSystem;
impl<'a> System<'a> for ApplyCollisionsSystem {
    type SystemData = (
        Option<Read<'a, ApplyCollisionsOption>>,
        ReadExpect<'a, Timestep>,
        ReadExpect<'a, CollisionParameters>,
        ReadExpect<'a, PartitionParameters>,
        WriteExpect<'a, CollisionsTracker>,
        WriteExpect<'a, VelocityHashmap>,
    );

    fn run(
        &mut self,
        (
            collisions_option,
            t,
            collision_params,
            partition_params,
            mut tracker,
            mut hashmap,
        ): Self::SystemData,
    ) {
        use rayon::prelude::*;

        match collisions_option {
            None => (),
            Some(_) => {
                // get immutable list of cells and iterate in parallel
                // (Note that using hashmap parallel values mut does not work in parallel, tested.)
                let cells: Vec<&mut PartitionCell> = hashmap.hashmap.values_mut().collect();
                println!("apply collisions running");
                println!("{}", cells.len());
                if cells.len() == 1 {
                    println!("{}", cells[0].particle_number);
                    println!("{}", cells[0].velocities[0]);
                    println!("{}", cells[0].velocities[1]);
                }
                cells.into_par_iter().for_each(|partition_cell| {
                    partition_cell.do_collisions(
                        partition_params.clone(),
                        collision_params.clone(),
                        t.delta,
                    );
                });

                tracker.num_atoms = hashmap
                    .hashmap
                    .values()
                    .map(|partition_cell| partition_cell.atom_number)
                    .collect();
                tracker.num_collisions = hashmap
                    .hashmap
                    .values()
                    .map(|partition_cell| partition_cell.collision_number)
                    .collect();
                tracker.num_particles = hashmap
                    .hashmap
                    .values()
                    .map(|partition_cell| partition_cell.particle_number)
                    .collect();
            }
        }
    }
}

fn do_collision<'a>(mut v1: Vector3<f64>, mut v2: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let mut rng = rand::thread_rng();

    // Randomly modify velocities in CoM frame, conserving energy & momentum
    let vcm = 0.5 * (v1 + v2);
    let energy: f64 = 0.5 * ((v1 - vcm).norm().powi(2) + (v2 - vcm).norm().powi(2));

    let cos_theta: f64 = rng.gen_range(-1.0..1.0);
    let sin_theta: f64 = (1.0 - cos_theta.powi(2)).sqrt();
    let phi: f64 = rng.gen_range(0.0..2.0 * PI);

    let v_prime = Vector3::new(
        energy.sqrt() * sin_theta * phi.cos(),
        energy.sqrt() * sin_theta * phi.sin(),
        energy.sqrt() * cos_theta,
    );
    v1 = vcm + v_prime;
    v2 = vcm - v_prime;

    (v1, v2)
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
    use crate::partition::BuildSpatialPartitionSystem;

    #[allow(unused_imports)]
    use nalgebra::Vector3;
    #[allow(unused_imports)]
    use specs::prelude::*;
    extern crate specs;

    #[test]
    fn test_do_collision() {
        // do this test muliple times since there is a random element involved in do_collision
        for _i in 0..50 {
            let v1 = Vector3::new(0.5, 1.0, 0.75);
            let v2 = Vector3::new(0.2, 0.0, 1.25);
            //calculate energy and momentum before
            let ptoti = v1 + v2;
            let energyi = 0.5 * (v1.norm_squared() + v2.norm_squared());

            let (v1new, v2new) = do_collision(v1, v2);

            //energy and momentum after
            let ptotf = v1new + v2new;
            let energyf = 0.5 * (v1new.norm_squared() + v2new.norm_squared());

            assert!((ptoti - ptotf) <= Vector3::new(1e-6, 1e-6, 1e-6));
            assert!((energyi - energyf) / energyi <= 1e-12);
            assert_ne!(v1, v1new);
            assert_ne!(v2, v2new);
        }
    }

    /// Test that the expected number of collisions in a CollisionBox is correct.
    #[test]
    fn collision_rate() {
        use assert_approx_eq::assert_approx_eq;

        let vel = Vector3::new(1.0, 0.0, 0.0);
        const MACRO_ATOM_NUMBER: usize = 100;
        let velocities: Vec<Velocity> = vec![Velocity { vel: vel.clone() }; MACRO_ATOM_NUMBER];
        let mut collision_box = PartitionCell::default();
        collision_box.velocities = velocities;

        let collision_params = CollisionParameters {
            macroparticle: 10.0,
            sigma: 1e-8,
            collision_limit: 10_000.0,
        };
        let partition_params = PartitionParameters {
            box_number: 1,
            box_width: 1e-3,
            target_density: 1.0,
        };
        let dt = 1e-3;
        collision_box.do_collisions(partition_params.clone(), collision_params.clone(), dt);
        assert_eq!(collision_box.particle_number, MACRO_ATOM_NUMBER as i32);
        let atom_number = collision_params.macroparticle * MACRO_ATOM_NUMBER as f64;
        assert_eq!(collision_box.atom_number, atom_number);
        let density = atom_number / partition_params.box_width.powi(3);
        let expected_number = (1.0 / SQRT2)
            * MACRO_ATOM_NUMBER as f64
            * density
            * collision_params.sigma
            * vel.norm()
            * dt;
        assert_approx_eq!(
            collision_box.expected_collision_number,
            expected_number,
            0.01
        );
    }

    /// Test that the system runs and causes nearby atoms to collide.
    #[test]
    fn test_collisions() {
        let mut test_world = World::new();

        ecs::register_components(&mut test_world);
        ecs::register_resources(&mut test_world);
        test_world.register::<NewlyCreated>();
        let mut atomecs_builder = AtomecsDispatcherBuilder::new();
        atomecs_builder.add_frame_initialisation_systems();
        atomecs_builder.add_systems();
        atomecs_builder
            .builder
            .add(BuildSpatialPartitionSystem, "build_partition", &[]);
        atomecs_builder
            .builder
            .add(ApplyCollisionsSystem, "collisions", &["build_partition"]);
        atomecs_builder.add_frame_end_systems();

        let builder = atomecs_builder.builder;
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut test_world);

        let vel1 = Vector3::new(1.0, 0.0, 0.0);
        let vel2 = Vector3::new(-1.0, 0.0, 0.0);

        let pos1 = Vector3::new(-3.0, 0.0, 0.0);
        let pos2 = Vector3::new(3.0, 0.0, 0.0);

        //atom 1 to collide
        let atom1 = test_world
            .create_entity()
            .with(Velocity { vel: vel1 })
            .with(Position { pos: pos1 })
            .with(Atom)
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(NewlyCreated)
            .build();

        //atom2 to collide
        let atom2 = test_world
            .create_entity()
            .with(Velocity { vel: vel2 })
            .with(Position { pos: pos2 })
            .with(Atom)
            .with(Force::new())
            .with(Mass { value: 87.0 })
            .with(NewlyCreated)
            .build();

        let dt = 1.0;
        test_world.insert(Timestep { delta: dt });
        test_world.insert(ApplyCollisionsOption);
        test_world.insert(CollisionsTracker {
            num_collisions: Vec::new(),
            num_atoms: Vec::new(),
            num_particles: Vec::new(),
        });
        test_world.insert(CollisionParameters {
            macroparticle: 1.0,
            sigma: 10.0,
            collision_limit: 10_000.0,
        });
        test_world.insert(PartitionParameters {
            box_number: 3,
            box_width: 2.0,
            target_density: 1.0,
        });
        test_world.insert(VelocityHashmap::default());

        for _i in 0..10 {
            dispatcher.dispatch(&mut test_world);
            test_world.maintain();
        }

        let velocities = test_world.read_storage::<Velocity>();
        let vel1new = velocities.get(atom1).expect("atom1 not found");
        let vel2new = velocities.get(atom2).expect("atom2 not found");

        let positions = test_world.read_storage::<Position>();
        let pos1new = positions.get(atom1).expect("atom1 not found");
        let pos2new = positions.get(atom2).expect("atom2 not found");

        assert_ne!(pos1, pos1new.pos);
        assert_ne!(pos2, pos2new.pos);

        assert_ne!(vel1 - vel1new.vel, Vector3::new(0.0, 0.0, 0.0));
        assert_ne!(vel2 - vel2new.vel, Vector3::new(0.0, 0.0, 0.0));
    }
}
