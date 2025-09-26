/// Wall collision logic. Does not work with forces/accelertions, or multiple entities.

use bevy::prelude::*;
use nalgebra::Vector3;
use rand::Rng;
use rand::distr::Distribution;
use crate::shapes::{
    Cylinder as MyCylinder,
    Cuboid as MyCuboid,
    Sphere as MySphere,
    CylindricalPipe,
    Volume
}; // Aliasing issues with bevy.
use crate::constant::{PI, EXP};
use crate::collisions::NumberOfWallCollisions;
use crate::atom::{Atom, Position, Velocity};
use crate::integrator::{Timestep, AtomECSBatchStrategy,};
use crate::initiate::NewlyCreated;
use crate::collisions::wall_collision_info::*;
use super::LambertianProbabilityDistribution;

#[derive(Resource)]
pub struct MaxSteps(pub i32); // Max steps for sdf convergence

#[derive(Resource)]
pub struct SurfaceThreshold(pub f64); // Tolerance for convergence

/// Struct to signify shape is a wall
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct WallData{
    pub wall_type: WallType,
}

/// Enum for designating wall type for collisions
pub enum WallType {
    // Specular
    Smooth,
    // Diffuse
    Rough,
    // choose randomly between specular and diffuse. Probability of specular reflection.
    Random {spec_prob: f64},
    // choose randomly depending on a coefficient that is exponential in angle of incidence.
    Exponential{value: f64},
}

#[derive(Component)]
pub struct DistanceToTravel {
    pub distance_to_travel: f64
}

#[derive(Component)]
pub enum VolumeStatus{
    Inside,
    Outside,
    Open,
}

pub struct CollisionInfo{
    pub atom: Entity,
    pub wall: Entity,
    pub collision_point: Vector3<f64>,
    pub collision_normal: Vector3<f64>
}

pub trait Wall {
    /// Collision check for open walls
    fn preliminary_collision_check(&self, 
        atom_pos: &Vector3<f64>, 
        atom_vel: &Vector3<f64>, 
        atom_location: &VolumeStatus,
        wall_pos: &Vector3<f64>, 
        dt: f64) -> bool;
}

impl<T : Volume> Wall for T {
    fn preliminary_collision_check(&self, 
        atom_pos: &Vector3<f64>, 
        _atom_vel: &Vector3<f64>, 
        atom_location: &VolumeStatus,
        wall_pos: &Vector3<f64>, 
        _dt: f64) -> bool {
        match atom_location {
            VolumeStatus::Inside => {
                return !self.contains(wall_pos, atom_pos);
            }
            VolumeStatus::Outside => {
                return self.contains(wall_pos, atom_pos);
            }
            VolumeStatus::Open => {
                return false;
            }
        }
    }
}

impl Wall for CylindricalPipe {
    fn preliminary_collision_check(&self, 
        atom_pos: &Vector3<f64>, 
        atom_vel: &Vector3<f64>, 
        _atom_location: &VolumeStatus,
        wall_pos: &Vector3<f64>, 
        dt: f64) -> bool {
        let delta_prev = (atom_pos - atom_vel * dt) - wall_pos;
        let axial_prev = delta_prev.dot(&self.direction);
        let radial_prev = delta_prev - axial_prev * self.direction;

        let delta_curr = atom_pos - wall_pos;
        let axial_curr = delta_curr.dot(&self.direction);
        let radial_curr = delta_curr - axial_curr * self.direction;

        let is_within_radius_prev = radial_prev.norm() <= self.radius;
        let is_within_length_prev = f64::abs(axial_prev) <= self.length / 2.0;
        let is_within_radius_curr = radial_curr.norm() <= self.radius;
        // let is_within_length_curr = f64::abs(axial_curr) <= self.length / 2.0;

        if is_within_radius_prev ^ is_within_radius_curr {
            if !is_within_length_prev {return false;}
            else {return true;}
        } else {
            return false;
        }
    }
}

/// Checks which atoms have collided.
fn collision_check<T: Wall + Intersect + Normal>(
    atom: (Entity, &Position, &Velocity, &VolumeStatus),
    wall: (Entity, &T, &Position),       
    dt: f64,
    tolerance: f64, 
    max_steps: i32,
) -> Option<CollisionInfo> {
    let (atom_entity, atom_pos, atom_vel, atom_location) = atom;
    let (wall_entity, shape, wall_pos) = wall;

    if !shape.preliminary_collision_check(&atom_pos.pos, &atom_vel.vel, atom_location, &wall_pos.pos, dt) {
        return None;
    }

    // Do collision check
    if let Some(mut collision_point) = shape.calculate_intersect(&atom_pos.pos, &atom_vel.vel, &wall_pos.pos, dt, tolerance, max_steps) {
        if let Some(collision_normal) = shape.calculate_normal(&collision_point, &wall_pos.pos, tolerance) {
            collision_point += 1e-10 * collision_normal; // Offset collision point along normal to avoid numerical issues
            return Some(CollisionInfo {
                atom: atom_entity,
                wall: wall_entity,
                collision_point,
                collision_normal,
            });
        } else {
            eprintln!(
                "Warning: Could not calculate normal. Atom: {:?}, Wall Pos: {:?}",
                atom_pos.pos, wall_pos.pos
            );
            None
        }
    } else {
        None
    }
}

/// Specular reflection
fn specular(
    collision_normal: &Vector3<f64>,
    velocity: &Vector3<f64>,
) -> Vector3<f64> {
    let reflected = velocity - 2.0 * velocity.dot(collision_normal) * collision_normal;
    reflected
}

/// Diffuse collision (Lambertian)
fn diffuse( 
    collision_normal: &Vector3<f64>, 
    velocity: &Vector3<f64>,
    distribution: &LambertianProbabilityDistribution) -> Vector3<f64> {
    // Get random weighted angles
    let mut rng = rand::rng();
    let phi = rng.random_range(0.0..2.0 * PI);
    let theta = distribution.sample(&mut rng);

    // Get basis vectors
    let dir = Vector3::new(65.514, 5.54, 41.5).normalize();
    let perp_x = collision_normal.cross(&dir).normalize();
    let perp_y = collision_normal.cross(&perp_x).normalize();
    
    // Get scattered velocity
    let x = phi.cos() * theta.sin();
    let y = phi.sin() * theta.sin();
    let z = theta.cos();

    (x * perp_x + y * perp_y + z * collision_normal) * velocity.norm()
}

// To be run once at startup
pub fn create_cosine_distribution(mut commands: Commands) {
    // tuple list of (theta, weight)
    let mut thetas = Vec::<f64>::new();
    let mut weights = Vec::<f64>::new();

    // precalculate the discretized cosine distribution.
    let n = 1000; // resolution over which to discretize `theta`.
    for i in 0..n {
        let theta = (i as f64) / (n as f64) * PI / 2.0;
        let weight = theta.cos() * theta.sin();
        thetas.push(theta);
        weights.push(weight);
        // Note: we can exclude d_theta because it is constant and the distribution will be normalized.
    }
    let cosine_distribution = LambertianProbabilityDistribution::new(thetas, weights);
    commands.insert_resource::<LambertianProbabilityDistribution>(cosine_distribution);
    println!("Cosine distribution created!")
}

/// Do wall collision
fn do_wall_collision(
    atom: (&mut Position, &mut Velocity, &mut DistanceToTravel, &mut NumberOfWallCollisions),
    wall: &WallData,
    collision: &CollisionInfo,
    distribution: &LambertianProbabilityDistribution,
    dt: f64,
    tolerance: f64,
) -> f64 {
    let (atom_pos, atom_vel, distance, num_of_collisions) = atom;
    num_of_collisions.value += 1;
    // subtract distance traveled to the collision point from prev position
    let traveled = ((atom_pos.pos - atom_vel.vel*dt) - collision.collision_point).norm();
    distance.distance_to_travel -= traveled;
    if - distance.distance_to_travel > tolerance {
        distance.distance_to_travel = 0.0;
        eprintln!("Distance to travel set to a negative value somehow");
    }
    // println!("original pos {}", (atom_pos.pos - atom_vel.vel*dt));

    // do collision
    match wall.wall_type {
        WallType::Smooth =>
            {atom_vel.vel = specular(&collision.collision_normal, &atom_vel.vel)}
        WallType::Rough =>
            {atom_vel.vel = diffuse(&collision.collision_normal, &atom_vel.vel, &distribution)}
        WallType::Random{spec_prob} => {
            if rand::rng().random_bool(spec_prob) {
                atom_vel.vel = specular(&collision.collision_normal, &atom_vel.vel)
            } else {
                atom_vel.vel = diffuse(&collision.collision_normal, &atom_vel.vel, &distribution)
            }
        }
        WallType::Exponential{value} => {
            let angle_of_incidence = (atom_vel.vel.dot(&collision.collision_normal)/atom_vel.vel.norm()).acos();
            let prob_spec = (EXP.powf(value * angle_of_incidence) - 1.0)/(EXP.powf(value * PI/2.0) - 1.0);
            if rand::rng().random_bool(prob_spec) {
                atom_vel.vel = specular(&collision.collision_normal, &atom_vel.vel)
            } else {
                atom_vel.vel = diffuse(&collision.collision_normal, &atom_vel.vel, &distribution)
            }
        } 
    }

    if distance.distance_to_travel > 0.0 {
        // propagate along chosen direction
        atom_pos.pos = collision.collision_point + atom_vel.vel.normalize() * distance.distance_to_travel;
    } else {
        atom_pos.pos = collision.collision_point;
    }
    // Return time used
    traveled / atom_vel.vel.norm()
}

/// Do the collisions
/// To be run after verlet integrate position
pub fn wall_collision_system<T: Wall + Component + Intersect + Normal>(
    wall_query: Query<(Entity, &T, &WallData, &Position), Without<Atom>>,
    mut atom_query: Query<(Entity, 
        &mut Position, 
        &mut Velocity, 
        &mut DistanceToTravel, 
        &mut NumberOfWallCollisions, 
        &VolumeStatus), 
        With<Atom>>,
    batch_strategy: Res<AtomECSBatchStrategy>,
    timestep: Res<Timestep>,
    threshold: Res<SurfaceThreshold>, 
    max_steps: Res<MaxSteps>,
    distribution: Res<LambertianProbabilityDistribution>,
) {
    let tolerance = threshold.0;
    let max_steps = max_steps.0;
    let dt = timestep.delta;
    let walls: Vec<_> = wall_query.iter().collect();
    atom_query
        .par_iter_mut()
        .batching_strategy(batch_strategy.0.clone())
        .for_each(|(atom_entity, mut pos, mut vel, mut distance, mut collisions, location)| {

            let mut dt_atom = dt;
            let mut num_of_collisions = 0;

            while distance.distance_to_travel > 0.0 && num_of_collisions < 10000 { 
                let mut collided = false;
                for (wall_entity, shape, wall, wall_pos) in &walls {
                    if let Some(collision_info) = collision_check(
                        (atom_entity, &pos, &vel, &location),
                        (*wall_entity, *shape, *wall_pos),
                        dt_atom,
                        tolerance,
                        max_steps,
                    ) {
                        // Handle the collision
                        let time_used = do_wall_collision((&mut pos, &mut vel, &mut distance, &mut collisions), 
                            *wall,
                            &collision_info, 
                            &distribution, 
                            dt_atom, 
                            tolerance);
                        collided = true;
                        dt_atom -= time_used;
                        if dt_atom < 0.0 {
                            dt_atom = 0.0;
                            // eprintln!("Time set to a negative value somehow, wall collision system");
                        }
                    }
                }
                
                if !collided {
                    break;
                }
                num_of_collisions += 1;
            }
        });
}

/// Updates distance to travel component for atoms
/// If an atom doesn't have DistanceToTravel, it will be initialized
/// Otherwise, it will be reset for the next frame
pub fn update_distance_to_travel_system(
    mut query_new: Query<(Entity, &Velocity), (With<Atom>, Without<DistanceToTravel>)>,
    mut query_existing: Query<(&Velocity, &mut DistanceToTravel), With<Atom>>,
    mut commands: Commands,
    timestep: Res<Timestep>,
) {
    // Initialize for new atoms
    for (atom_entity, vel) in query_new.iter_mut() {
        commands.entity(atom_entity).insert(DistanceToTravel {
            distance_to_travel: vel.vel.norm() * timestep.delta
        });
    }

    // Reset for existing atoms
    for (vel, mut distance) in query_existing.iter_mut() {
        distance.distance_to_travel = vel.vel.norm() * timestep.delta;
    }
}

/// Assign location status to atoms
pub fn assign_location_status_system(
    mut query: Query<(Entity, &Position), (With<Atom>, With<NewlyCreated>)>,
    spheres: Query<(&MySphere, &Position), With<WallData>>,
    cylinders: Query<(&MyCylinder, &Position), With<WallData>>,
    cuboids: Query<(&MyCuboid, &Position), With<WallData>>,
    mut commands: Commands,
) {
    for (atom_entity, pos) in query.iter_mut() {
        let has_walls = spheres.iter().count() > 0 || 
                        cylinders.iter().count() > 0 || 
                        cuboids.iter().count() > 0;

        let inside = spheres.iter().any(|(wall, wall_pos)| wall.contains(&wall_pos.pos, &pos.pos)) ||
                     cylinders.iter().any(|(wall, wall_pos)| wall.contains(&wall_pos.pos, &pos.pos)) ||
                     cuboids.iter().any(|(wall, wall_pos)| wall.contains(&wall_pos.pos, &pos.pos));

        let status = if has_walls {
            if inside {
                VolumeStatus::Inside
            } else {
                VolumeStatus::Outside
            }
        } else {
            // No closed walls, 
            VolumeStatus::Open
        };

        commands.entity(atom_entity).insert(status);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom::{Atom, Position, Velocity};
    use crate::shapes::{
        Cylinder as MyCylinder,
        Cuboid as MyCuboid,
        Sphere as MySphere,
    };
    use assert_approx_eq::assert_approx_eq;

    const TOLERANCE: f64 = 1e-9;
    const MAX_STEPS: i32 = 1000;

    #[test]
    fn test_collision_check() {
        let mut app = App::new();
        
        // Helper function to test collisions
        fn test_collision<T>(
            app: &mut App,
            atom_pos: Vector3<f64>,
            wall_pos: Vector3<f64>,
            wall: T,
            expected_point: Vector3<f64>,
        ) where T: Component + Wall + Intersect + Normal {
            let velocity = Velocity { vel: Vector3::new(1.0, 0.0, 0.0) };
            let position = Position { pos: atom_pos };
            let dt = 1.0 + 1e-15;

            let atom = app.world_mut().spawn(position.clone())
                .insert(velocity.clone())
                .insert(Atom)
                .insert(VolumeStatus::Inside)
                .id();

            let wall_position = Position { pos: wall_pos };
            let wall_entity = app.world_mut().spawn(wall_position.clone())
                .insert(WallData { wall_type: WallType::Smooth })
                .insert(wall)
                .id();

            let wall_component = app.world().entity(wall_entity).get::<T>().unwrap();

            let result = collision_check(
                (atom, &position, &velocity, &VolumeStatus::Inside),
                (wall_entity, wall_component, &wall_position),
                dt, TOLERANCE, MAX_STEPS
            );

            match result {
                Some(collision) => {
                    assert_eq!(atom, collision.atom);
                    assert_eq!(wall_entity, collision.wall);
                    for i in 0..3 {
                        assert_approx_eq!(expected_point[i], collision.collision_point[i], TOLERANCE);
                        assert_approx_eq!([-1.0, 0.0, 0.0][i], collision.collision_normal[i], 1e-14);
                    }
                }
                None => panic!("Collision not detected!"),
            }

            app.world_mut().despawn(atom);
            app.world_mut().despawn(wall_entity);
        }


        // Test cases for different wall types
        test_collision(
            &mut app,
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            MySphere { radius: 4.0 },
            Vector3::new(4.0, 0.0, 0.0),
        );

        test_collision(
            &mut app,
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            MyCuboid { half_width: Vector3::new(4.0, 1.0, 1.0) },
            Vector3::new(4.0, 0.0, 0.0),
        );

        test_collision(
            &mut app,
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            MyCylinder::new(1.0, 8.0, Vector3::new(1.0, 0.0, 0.0)),
            Vector3::new(4.0, 0.0, 0.0),
        );

        test_collision(
            &mut app,
            Vector3::new(2.5, 1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0),
            CylindricalPipe::new(1.0, 8.0, Vector3::new(0.0, 1.0, 0.0)),
            Vector3::new(2.0, 1.0, 1.0),
        );
    }

    // this is ugly as sin
    #[test]
    fn test_do_wall_collision() {
        let mut app = App::new();
        let dt = 1.0 - 1e-10;
        let position = Position {pos: Vector3::new(1.0, 1.0, 0.0)};
        let velocity = Velocity {vel: Vector3::new(2.0,2.0,0.0)};
        let length = velocity.vel.norm() * dt;
        let distance = DistanceToTravel{distance_to_travel: length};
        let atom = app.world_mut()
            .spawn(Atom)
            .insert(position.clone())
            .insert(velocity.clone())
            .insert(DistanceToTravel{distance_to_travel: length})
            .insert(NumberOfWallCollisions{value: 0})
            .insert(VolumeStatus::Inside)
            .id();
        app.world_mut().spawn(WallData {wall_type: WallType::Smooth});
        let collision_point = Vector3::new(0.0,0.0,0.0);

        fn test_system(
            mut query: Query<(Entity, &mut Position, &mut Velocity, &mut DistanceToTravel, &mut NumberOfWallCollisions)>,
            wall: Query<(Entity, &WallData)>,
            distribution: Res<LambertianProbabilityDistribution>,
            tolerance: Res<SurfaceThreshold>) {
            query.iter_mut().for_each(|(atom, mut atom_pos, mut atom_vel, mut distance, mut collisions)| {
                for (wall_entity, wall) in wall.iter() {
                    let dt = 1.0 - 1e-10; // If you change this make sure to change the dt above as well
                    let collision_point = Vector3::new(0.0,0.0,0.0);
                    let collision_normal = Vector3::new(-1.0,0.0,0.0);
                    let collision_info = CollisionInfo {
                        atom,
                        wall: wall_entity,
                        collision_point,
                        collision_normal,
                    };
                    let time_used = do_wall_collision((&mut atom_pos, &mut atom_vel, &mut distance, &mut collisions), 
                        wall,  
                        &collision_info, 
                        &distribution, 
                        dt, 
                        tolerance.0);
                    assert_approx_eq!(time_used, ((atom_pos.pos - atom_vel.vel*dt) - collision_point).norm()/atom_vel.vel.norm())
                }
            });
        }
        app.add_systems(Startup, create_cosine_distribution);
        app.add_systems(Update, test_system);
        app.world_mut().insert_resource(SurfaceThreshold(1e-9));
        app.update();

        let new_velocity = app.world()
            .entity(atom)
            .get::<Velocity>()
            .expect("Entity not found!")
            .vel;
        let new_position = app.world()
            .entity(atom)
            .get::<Position>()
            .expect("Entity not found")
            .pos;
        let new_distance = app.world()
            .entity(atom)
            .get::<DistanceToTravel>()
            .expect("Entity not found")
            .distance_to_travel;
        let new_number_of_wall_collisions = app.world()
            .entity(atom)
            .get::<NumberOfWallCollisions>()
            .expect("Entity not found")
            .value;
            
        let time_used = ((position.pos - velocity.vel*dt) - collision_point).norm()/velocity.vel.norm();
        let distance_travelled = ((position.pos - velocity.vel*dt) - collision_point).norm();

        assert_ne!(new_velocity, velocity.vel);
        assert_eq!(new_number_of_wall_collisions, 1);
        assert_approx_eq!(new_position[0], collision_point[0] + new_velocity[0]*(dt - time_used), 1e-15);
        assert_approx_eq!(new_position[1], collision_point[1] + new_velocity[1]*(dt - time_used), 1e-15);
        assert_approx_eq!(new_position[2], collision_point[2] + new_velocity[2]*(dt - time_used), 1e-15);
        assert_approx_eq!(distance.distance_to_travel - distance_travelled, new_distance, 1e-15);
    }

    // Test multiple collisions
    // Basically an integration test
    #[test]
    fn test_systems() {
        use crate::integrator::{IntegrationPlugin, Timestep, OldForce};
        use crate::initiate::NewlyCreated;
        use crate::atom::{Force, Mass};
        use crate::collisions::ApplyAtomCollisions;
        use crate::collisions::CollisionPlugin;

        let mut app = App::new();
        app.world_mut()
            .spawn(WallData{wall_type: WallType::Smooth})
            .insert(CylindricalPipe::new(1.0, 8.0, Vector3::new(0.0, 0.0, 1.0)))
            .insert(Position{pos: Vector3::new(0.0, 0.0, 0.0)});
        let atom = app.world_mut()
            .spawn(Atom)
            .insert(Position{pos: Vector3::new(-1.0 + 1e-8, 0.0, 0.0)})
            .insert(Velocity{vel: Vector3::new(0.0, -2.5 * PI, 0.0)})
            .insert(NewlyCreated)
            .insert(Force::default())
            .insert(Mass { value: 1.0 })
            .insert(OldForce(Force::default()))
            .id();
        app.add_plugins(IntegrationPlugin);
        app.add_plugins(CollisionPlugin);
        app.world_mut().insert_resource(ApplyAtomCollisions(false));
        app.world_mut().insert_resource(Timestep {delta: 1.0});
        app.update();

        let new_position = app.world()
            .entity(atom)
            .get::<Position>()
            .expect("Entity not found")
            .pos;

        let wall_collisions = app.world()
            .entity(atom)
            .get::<NumberOfWallCollisions>()
            .expect("Entity not found")
            .value;

        println!("{}", new_position);
        println!("wall collisions {}", wall_collisions); 
        assert_approx_eq!(new_position[0], 0.0, 1e-5);
        assert_approx_eq!(new_position[1], -1.0, 1e-5);
        assert_approx_eq!(new_position[2], 0.0, 1e-5);
    }
}


