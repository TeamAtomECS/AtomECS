# rustmot

`rustmot` is a crate that simulates the laser-cooling of atoms, and supports numerous features:
* Doppler forces on atoms that scatter light, including (optionally) the random fluctuations that give rise to the Doppler temperature limit.
* Magnetic fields, implemented on a grid or through simple analytical models.
* Atoms generated on the surface of a chamber.
* Atoms generated from an oven source.
* Cooling light defined by detuning and gaussian profiles.
* Volumes to define bounds for the simulation.
* File output to binary or text files.
* Thorough unit testing to ensure simulation results are correct.

Additionally, simulations may be wrapped using python/matlab, as shown in the [optimot](https://bitbucket.org/footgroup/optimot) repo.

If you would like to get started, try some examples with `cargo run --release --example 1d_mot`, then use the scripts in the Matlab directory to plot the results.
You can also use `cargo doc` to explore the documentation, which has more detail on the structure of the program.

`rustmot` is written in the Entity-Component-System (ECS) pattern, implemented using [specs](https://github.com/slide-rs/specs).
If you are unfamiliar with this pattern, it is thoroughly recommended that you read about it before diving into the code.

`rustmot` is written by Xuhui Chen & Elliot Bentine.

### Current Limitations

* atom-atom interactions are not implemented. Most of our work deals with atom sources, which have low steady-state number densities, so we haven't implemented this. Results for steady-state 3D MOTs should be interpreted carefully.
