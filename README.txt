Content:
------------- Modules

------------- Important entities

------------- Systems and their executing order



Modules:

constant: Maths and Physical constant

maths: maths function used in the program, should be noted that all the array operation are for 3 dimensional array only

integration: a library for integrating the path of individual atom

ouput: a module that print the necessary output of the simulation both for file output and console ouput, Detectors are also in this module

magnetic: everything about magnetic field in the simulation is in this module

laser: everything about lasers and their interactions with atom is in this module

atom_create: module for system and component necessary for the creating of atoms in the simulation

atom : general Component for an atom

initiate: some iniatialization information

ecs: a module that allow registering resources and creating systems in simple functions

simulation_templates: well, simulation templates

----------------------------------------------------------------

Important Entities:

AtomInfo: a template for an atom

Atom: marker that indicates that this entity is an atom

MagneticSampler: a component that indicate the magnetic field that is currently present on the entity

Position/ Velocity / Mass : no need for any explanation, need to note that 3D space is assumed

Detector/ RingDetector: will detect entities that get into their domain, remove their Position/ Velocity Component and store the relevant data

NewlyCreated: a marker that indicate that this entity is newly created and initializors should work on it


--------------------------------------------------------------------

Systems and their executing order:

In short, just look at the ecs modules, the executing order is there.

More detailed explanation:

Laser Systems must be exectuing after the Magnetic ones, DeflagSystem must be ran after all initializors so that an entity would be initialize twice.
Atom creation system must be ran in the very beginning, and the output module must be ran in the end for obvious reason.

Output of the detector can only be ran after all other systems have been ran.


