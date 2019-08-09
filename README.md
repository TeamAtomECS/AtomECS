# rustmot

Simulate laser-cooled atoms using rust and `specs`.

_Written by Xuhui Chen and Elliot Bentine_
____________________________________

## Intro

The program is structured using the Entity-Component-System (ECS) design pattern, implemented using the [specs](https://github.com/slide-rs/specs) crate.
Functionality is stored throughout several modules, as detailed below.

## Modules:

| Module      | Description |
|-------------|-------------|
|`constant`   | Lists physical constants used by the program. |
|`maths`      | Maths functions used in the program. |
|`integrator`| Used to integrate the equations of motion to update positions and velocities of entities. |
|`ouput`      | Systems and components used to generate output files from the simulation, eg atomic positions or console output. |
|`magnetic`   | Implements different magnetic fields. |
|`laser`      | Implements optical scattering forces and lasers. |
|`oven`| Systems and components used to create atoms in the simulation. |
|`atom`       | All components relating to atoms. |
|`initiate`   | Components and systems used to initiate entities during the simulation. |
|`ecs`        | Easy functions to setup the simulation dispatcher and world resources. |
|`simulation_templates`| Well, simulation templates... |

## Components:

We outline a few of the key components here.

| Component            | Description |
|----------------------|-------------|
|`AtomInfo`            | Container for a number of properties related to atomic species, eg frequency of laser cooling transition, linewidth, etc. |
|`Atom`                | A marker that indicates that this entity is an atom. |
|`MagneticSampler`     | A component that directs the `magnetic` Systems to calculate magnetic fields at the location of this entity. |
|`Position`, `Velocity`, `Mass` | No need for any explanation. |
|`Detector`,`RingDetector` | These detectors count the number of atoms that enter a defined region. The detector systems delete the atoms and store the relevant data. |
|`NewlyCreated`        | A marker that indicates an entity is newly created. This signals to other modules to initialize required components. The marker is removed by the `DeflagSystem`. |


## Systems

TODO

### Execution Order

* `laser` Systems execute after the `magnetic` ones.
* `DeflagSystem` is used to remove the `NewlyCreated` component. The removal is done through a `specs::LazyUpdate`, so it is actually enacted at the end of the frame. As such, the order with respect to initialiser systems is not important.
* The `output` modules should run at the end of the frame so that generated output reflects the state of the frame (and doesn't occur half-way through an update). This also includes the detectors.

### Current Limitations

* atom-atom interactions are ignored. This isn't a problem for the 2D MOT that we want to simulate, but it is going to be incorrect for 3D MOT simulations which achieve higher steady-state densities.