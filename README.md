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
|`simulation_templates`| Well, it is simulation templates, one of the simulation templates is load_from_config which will create the simulation based on the yaml file|
|`example`| A few running example of the program|
|`destructor`| A module reponsible for destroying the atoms and decide if an atom is out of bound 



## Components:

We outline a few of the key components here.

| Component            | Description |
|----------------------|-------------|
|`AtomInfo`            | Container for a number of properties related to atomic species, eg frequency of laser cooling transition, linewidth, etc. |
|`Atom`                | A marker that indicates that this entity is an atom. |
|`MagneticSampler`     | A component that directs the `magnetic` Systems to calculate magnetic fields at the location of this entity. |
|`Position`, `Velocity`, `Mass` | No need for any explanation. |
|`Detector`,`RingDetector` | These detectors count the number of atoms that enter a defined region. The detector systems delete the atoms and store the relevant data. |
|`NewlyCreated`         | A marker that indicates an entity is newly created. This signals to other modules to initialize required components. The marker is removed by the `DeflagSystem`.|
|`ToBeDestroyed `       | A marker that indicate that the entity needs to be detroyed the next frame. DO NOTE that though it can be very convenient and useful at times, it should NOT Be used when an entity need to be detroyed immediately |
|`SimArchetype`,`...Archetype`| information used to generate the simulations.|
|`Detected`                 | A marker that indicated that the atom is currently in the detecting region, once the atom is inside the detecting region for a user-specified amount of time, the time will be considered "captured" |



## Important Systems

|Systems in `Laser` module| (can be registered easily using laser.add_system_to_dispatch) Those systems calculate the forces and assign the details to each atoms. Need to be run after the magnetic systems. It will also index the lasers and record the interaction between different lasers and an atom in cooling_force component (the info is recorded in the order of the laser)

|Systems in `magnetic` module| (can be registered using magnetic.add_system_to_dispatch) These systems calculate the magnetic field at the position of each individual atoms. Different type of magnetic field can be used.

|`Random_Walk_System`| Including the effect of random walk due to emission of photon. in this system only random walk of the size of a photon will be included.

|`DestoyedOutofBoundAtomSystem`| destroyed the atoms that hit the walls, the bound need to be set manually.

|`DetectingAtomSystem`| The system that detect the atoms and print the information the file corresponding to the detectors

|`FileOutputSystem`| The system that print the information of atom every a number of frames

|`ConsoleOutputSystem`| The system that print the information of the atom very 1000 (can be customized) frames

|`DestroyOutOfBoundAtomsSystem`| The system that detects the atoms that are out of bound, the bound can be specified 


### Execution Order

* `laser` Systems execute after the `magnetic` ones.
* `DeflagSystem` is used to remove the `NewlyCreated` component. The removal is done through a `specs::LazyUpdate`, so it is actually enacted at the end of the frame. As such, the order with respect to initialiser systems is not important.
* The `output` modules should run at the end of the frame so that generated output reflects the state of the frame (and doesn't occur half-way through an update). This also includes the detectors.

### Current Limitations

* atom-atom interactions are ignored. This isn't a problem for the 2D MOT that we want to simulate, but it is going to be incorrect for 3D MOT simulations which achieve higher steady-state densities.

* Choices of Oven types as well as some other choices (e.x. shape of the wall) cannot be made using hte config file. A new "world" need to be created manually if all functionality of the program need to be used.

# Users guide

## execute from file

* load_from_config function can create a world (exp setup) based on the a simple yaml file, an example is given in the crate as example.yaml. Multiple lasers and ovens are allowed.

* for the oven part of the file, rate indicate the number of atom emitted per second while instand_emission indicate the number of atom produced at the beginning of the simulation. both can be used at the same time.

## execute from rust program

* Some templates exist in simulation templates file, they can be ran to create the world (exp setup).

* A customised design can be created by just changing the templates.

## resources that need to be registered manually

* RandomWalkMarker indicates if random walk needs to be included, BoundaryMarker indicates if boundary( walls ) needs to be included. ( the shape of the boudnary still need to be changed manually in the program). 

* VelocityCap is the velocity cap for all atom sources. All atoms that goes beyond the cap will be discarded immediately to save the computational power.

* RepumpLoss indicate the proportion of the atoms that will be lost (not longer interact with the laser) during emission of one photon. 

* OptEarly, a Component that indicate that the simulation will be optimized at the cost of the accuracy. If used, the simulation timestep will be doubled at the begining of the simulation when the atoms are barely interacting with the lasers.

## parts that need to be designed manually

* Shape of the boundary
Can be changed manually at detructor::out_of_bound function. This part can only be changed manually as the shaoe of the allowed region cannot be specified simply by a few parameters.

* Shape of the oven aperture (optional)
The default shape is circular, but cubic choice is also available and can be registered manually.

* FileOutputSystem, you can decide what you want as the output
The default file output will be the velocity and the position of the atoms. If some other information is needed, output::fileoutput::FileOutputSystem can be changed.
