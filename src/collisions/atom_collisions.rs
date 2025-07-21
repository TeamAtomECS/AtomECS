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
use crate::atom::{Position, Velocity, Atom};
use crate::constant::{PI, SQRT2};
use crate::integrator::Timestep;
use hashbrown::HashMap;
use nalgebra::Vector3;
use rand::Rng;
use bevy::prelude::*;
use crate::integrator::IntegrationSet;
use crate::integrator::AtomECSBatchStrategy;

/// A resource that indicates that the simulation should apply scattering
#[derive(Resource)]
pub struct ApplyCollisionsOption{
    pub apply_collision: bool,
}

/// Component that marks which box an atom is in for spatial partitioning
#[derive(Component)]
pub struct BoxID {
    /// ID of the box
    pub id: i64,
}

/// A patition of space within which collisions can occur
pub struct CollisionBox<> {
    pub entity_velocities: Vec<(Entity, Vector3<f64>)>,
    pub expected_collision_number: f64,
    pub collision_number: i32,
    pub density: f64,
    pub volume: f64,
    pub atom_number: f64,
    pub particle_number: i32,
}

impl Default for CollisionBox {
    fn default() -> Self {
        CollisionBox {
            entity_velocities: Vec::new(),
            expected_collision_number: 0.0,
            density: 0.0,
            volume: 0.0,
            atom_number: 0.0,
            collision_number: 0,
            particle_number: 0,
        }
    }
}

impl CollisionBox {
    /// Perform collisions within a box.
    fn do_collisions(&mut self, params: CollisionParameters, dt: f64) {
        let mut rng = rand::rng();
        self.particle_number = self.entity_velocities.len() as i32;
        self.atom_number = self.particle_number as f64 * params.macroparticle;

        // Only one atom or less in box - no collisions.
        if self.particle_number <= 1 {
            return;
        }

        ///// n*sigma*v total collisions
        // vbar is the average _speed_, not the average _velocity_.
        let mut vsum = 0.0;
        for (_, velocity) in &self.entity_velocities {
            vsum += velocity.norm();
        }
        let vbar = vsum / self.entity_velocities.len() as f64;

        // number of collisions is N*n*sigma*v*dt, where n is atom density and N is atom number
        // probability of one particle colliding is n*sigma*vrel*dt where n is the atom density, sigma cross section and vrel the average relative velocity
        // vrel = SQRT(2)*vbar, and since we assume these are identical particles we must divide by two since otherwise we count each collision twice
        // so total number of collisions is N_particles * probability = N_p*n*sigma*vbar*dt/SQRT(2)
        let density = self.atom_number / params.box_width.powi(3);
        self.expected_collision_number =
            self.particle_number as f64 * density * params.sigma * vbar * dt * (1.0 / SQRT2);

        let mut num_collisions_left: f64 = self.expected_collision_number;

        if num_collisions_left > params.collision_limit {
            panic!("Number of collisions in a box in a single frame exceeds limit. Number of collisions={}, limit={}, particles={}.", num_collisions_left, params.collision_limit, self.particle_number);
        }

        while num_collisions_left > 0.0 {
            let collide = if num_collisions_left > 1.0 {
                true
            } else {
                rng.random::<f64>() < num_collisions_left
            };

            if collide {
                let idx1 = rng.random_range(0..self.entity_velocities.len());
                let mut idx2 = idx1;
                while idx2 == idx1 {
                    idx2 = rng.random_range(0..self.entity_velocities.len())
                }

                let v1 = self.entity_velocities[idx1].1;
                let v2 = self.entity_velocities[idx2].1;
                let (v1new, v2new) = do_collision(v1, v2);
                self.entity_velocities[idx1].1 = v1new;
                self.entity_velocities[idx2].1 = v2new;
                self.collision_number += 1;
            }

            num_collisions_left -= 1.0;
        }
        /////
    }
}

/// Resource for defining collision relevant paramaters like macroparticle number, box width and number of boxes
#[derive(Copy, Clone, Resource)]
pub struct CollisionParameters {
    /// number of real particles one simulation particle represents for collisions
    pub macroparticle: f64,
    /// number of boxes per side in spatial binning
    pub box_number: i64,
    /// width of one box in m
    pub box_width: f64,
    // collisional cross section of atoms (assuming only one species)
    pub sigma: f64,
    /// Limit on number of collisions per box each frame. If the number of collisions to calculate exceeds this, the simulation will panic.
    pub collision_limit: f64,
}

/// Resource that stores stats about collisions
#[derive(Clone, Resource)]
pub struct CollisionsTracker {
    /// number of collisions in each box
    pub num_collisions: Vec<i32>,
    /// number of simulated particles in each box
    pub num_particles: Vec<i32>,
    /// number of simulated atoms in each box
    pub num_atoms: Vec<f64>,
}

/// Initializes boxid component
pub fn init_boxid_system(
    mut query: Query<(Entity), (Without<BoxID>, With<Atom>)>,
    collisions_option: Res<ApplyCollisionsOption>,
    mut commands: Commands,
) {
    match collisions_option.apply_collision {
        false => return, // No need to initialize box IDs if collisions are not applied
        true => {
            query
                .iter_mut()
                .for_each(|(entity)| {
                    commands.entity(entity).insert(BoxID { id: 0 });
                });},
    }
}

/// Performs collisions within the atom cloud using a spatially partitioned Monte-Carlo approach.
pub fn apply_collisions_system(
    mut query: Query<(Entity, &Position, &mut Velocity, &mut BoxID), With<Atom>>,
    collisions_option: Res<ApplyCollisionsOption>,
    timestep: Res<Timestep>,
    params: Res<CollisionParameters>,
    mut tracker: ResMut<CollisionsTracker>,
    batch_strategy: Res<AtomECSBatchStrategy>,
) {
    use rayon::prelude::*;
    match collisions_option.apply_collision {
        false => (),
        true => {
            // Assign box IDs to atoms
            query
                .par_iter_mut()
                .batching_strategy(batch_strategy.0.clone())
                .for_each(|(_, position, _, mut boxid)| {
                    boxid.id = pos_to_id(position.pos, params.box_number, params.box_width);
                });

            // insert atom velocity into hash
            let mut map: HashMap<i64, CollisionBox> = HashMap::new();
            for (entity, _, velocity, boxid) in query.iter() {
                if boxid.id == i64::MAX {
                    continue;
                } else {
                    map.entry(boxid.id).or_default()
                        .entity_velocities.push((entity, velocity.vel));
                }
            }

            // get immutable list of boxes and iterate in parallel
            // (Note that using hashmap parallel values mut does not work in parallel, tested.)
            let boxes: Vec<&mut CollisionBox> = map.values_mut().collect();
            boxes.into_par_iter().for_each(|collision_box| {
                collision_box.do_collisions(*params, timestep.delta);
            });

            // Write back values.
            for collision_box in map.values() {
                for &(entity, new_velocity) in &collision_box.entity_velocities {
                    if let Ok((_, _, mut velocity, _)) = query.get_mut(entity) {
                        velocity.vel = new_velocity;
                    }
                }
            }

            // Update tracker
            tracker.num_atoms = map
                .values()
                .map(|collision_box| collision_box.atom_number)
                .collect();
            tracker.num_collisions = map
                .values()
                .map(|collision_box| collision_box.collision_number)
                .collect();
            tracker.num_particles = map
                .values()
                .map(|collision_box| collision_box.particle_number)
                .collect();
        }
    }
}


fn do_collision(mut v1: Vector3<f64>, mut v2: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let mut rng = rand::rng();

    // Randomly modify velocities in CoM frame, conserving energy & momentum
    let vcm = 0.5 * (v1 + v2);
    let energy: f64 = 0.5 * ((v1 - vcm).norm().powi(2) + (v2 - vcm).norm().powi(2));

    let cos_theta: f64 = rng.random_range(-1.0..1.0);
    let sin_theta: f64 = (1.0 - cos_theta.powi(2)).sqrt();
    let phi: f64 = rng.random_range(0.0..2.0 * PI);

    let v_prime = Vector3::new(
        energy.sqrt() * sin_theta * phi.cos(),
        energy.sqrt() * sin_theta * phi.sin(),
        energy.sqrt() * cos_theta,
    );
    v1 = vcm + v_prime;
    v2 = vcm - v_prime;

    (v1, v2)
}

fn pos_to_id(pos: Vector3<f64>, n: i64, width: f64) -> i64 {
    //Assume that atoms that leave the grid are too sparse to collide, so disregard them
    //We'll assign them the max value of i64, and then check for this value when we do a collision and ignore them
    let bound = (n as f64) / 2.0 * width;

    let id: i64;
    if pos[0].abs() > bound || pos[1].abs() > bound || pos[2].abs() > bound {
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
    use crate::initiate::NewlyCreated;
    #[allow(unused_imports)]
    use crate::integrator::{
        Step, Timestep,
    };
    use crate::collisions::CollisionPlugin;

    #[allow(unused_imports)]
    use nalgebra::Vector3;
    #[allow(unused_imports)]
    use bevy::prelude::*;
    extern crate bevy;
    use crate::simulation::SimulationBuilder;

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
    fn test_do_collision() {
        // do this test muliple times since there is a random element involved in do_collision
        let v1 = Vector3::new(0.5, 1.0, 0.75);
        let v2 = Vector3::new(0.2, 0.0, 1.25);
        //calculate energy and momentum before
        let ptoti = v1 + v2;
        let energyi = 0.5 * (v1.norm_squared() + v2.norm_squared());
        for _i in 0..50 {
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
        
        // Create dummy entities for testing
        let mut entity_velocities: Vec<(Entity, Vector3<f64>)> = Vec::new();
        for i in 0..MACRO_ATOM_NUMBER {
            let entity = Entity::PLACEHOLDER; // Or just use the same placeholder for all
            entity_velocities.push((entity, vel));
        }
        
        let mut collision_box = CollisionBox {
            entity_velocities,
            ..Default::default()
        };
        
        let params = CollisionParameters {
            macroparticle: 10.0,
            box_number: 1,
            box_width: 1e-3,
            sigma: 1e-8,
            collision_limit: 10_000.0,
        };
        let dt = 1e-3;
        collision_box.do_collisions(params, dt);
        
        assert_eq!(collision_box.particle_number, MACRO_ATOM_NUMBER as i32);
        let atom_number = params.macroparticle * MACRO_ATOM_NUMBER as f64;
        assert_eq!(collision_box.atom_number, atom_number);
        let density = atom_number / params.box_width.powi(3);
        let expected_number =
            (1.0 / SQRT2) * MACRO_ATOM_NUMBER as f64 * density * params.sigma * vel.norm() * dt;
        assert_approx_eq!(
            collision_box.expected_collision_number,
            expected_number,
            0.01
        );
    }


    /// Test that the system runs and causes nearby atoms to collide. More of an integration test than a unit test.
    #[test]
    fn test_collisions() {
        let mut simulation_builder = SimulationBuilder::default();
        // simulation_builder.add_end_frame_systems();
        simulation_builder.add_plugins(CollisionPlugin);
        let mut sim = simulation_builder.build();

        let vel1 = Vector3::new(1.0, 0.0, 0.0);
        let vel2 = Vector3::new(-1.0, 0.0, 0.0);

        let pos1 = Vector3::new(-3.0, 0.0, 0.0);
        let pos2 = Vector3::new(3.0, 0.0, 0.0);

        //atom 1 to collide
        let atom1 = sim
            .world_mut()
            .spawn(Velocity { vel: vel1 })
            .insert(Position { pos: pos1 })
            .insert(Atom)
            .insert(Force::default())
            .insert(Mass { value: 87.0 })
            .insert(NewlyCreated)
            .id();

        //atom2 to collide
        let atom2 = sim
            .world_mut()
            .spawn(Velocity { vel: vel2 })
            .insert(Position { pos: pos2 })
            .insert(Atom)
            .insert(Force::default())
            .insert(Mass { value: 87.0 })
            .insert(NewlyCreated)
            .id();

        let dt = 1.0;
        sim.world_mut().insert_resource(Timestep { delta: dt });
        sim.world_mut().insert_resource(ApplyCollisionsOption{apply_collision: true});
        sim.world_mut().insert_resource(CollisionsTracker {
            num_collisions: Vec::new(),
            num_atoms: Vec::new(),
            num_particles: Vec::new(),
        });
        sim.world_mut().insert_resource(CollisionParameters {
            macroparticle: 1.0,
            box_number: 10,
            box_width: 2.0,
            sigma: 10.0,
            collision_limit: 10_000.0,
        });

        for _i in 0..10 {
            sim.update();
        }

        let vel1new = sim.world().entity(atom1).get::<Velocity>().expect("atom1 not found");
        let vel2new = sim.world().entity(atom2).get::<Velocity>().expect("atom2 not found");

        let pos1new = sim.world().entity(atom1).get::<Position>().expect("atom1 not found");
        let pos2new = sim.world().entity(atom2).get::<Position>().expect("atom2 not found");

        assert_ne!(pos1, pos1new.pos);
        assert_ne!(pos2, pos2new.pos);

        assert_ne!(vel1 - vel1new.vel, Vector3::new(0.0, 0.0, 0.0));
        assert_ne!(vel2 - vel2new.vel, Vector3::new(0.0, 0.0, 0.0));
    }
}
