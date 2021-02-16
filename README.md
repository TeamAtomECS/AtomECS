# AtomECS

`atomecs` is a crate that simulates the laser-cooling of atoms, and supports numerous features:
* Doppler forces on atoms that scatter light, including (optionally) the random fluctuations that give rise to the Doppler temperature limit.
* Magnetic fields, implemented on a grid or through simple analytical models.
* Atoms generated on the surface of a chamber.
* Atoms generated from an oven source.
* Cooling light defined by detuning and gaussian profiles.
* Volumes to define bounds for the simulation.
* File output to binary or text files.
* Thorough unit testing to ensure simulation results are correct.

Additionally, simulations may be wrapped using python/matlab, as shown in the [AIONSource](https://github.com/TeamAtomECS/AIONSource) repo.

## Getting Started

If you would like to get started, try some examples with `cargo run --release --example 1d_mot`, then use the scripts in the Matlab directory to plot the results.
You can also use `cargo doc` to explore the documentation, which has more detail on the structure of the program.

## Development notes

`atomecs` is written in the Entity-Component-System (ECS) pattern, implemented using [specs](https://github.com/slide-rs/specs) for rust.
ECS is a data-oriented pattern that is well suited to high performance simulations, and is flexible enough to accomodate changing design goals.
For these reasons, ECS has become established in the video game industry, and since 2018 Unity (one of the market leaders) has embraced the pattern.
_If you are unfamiliar with this pattern, it is thoroughly recommended that you read about it before diving into the code._
Some useful links are:
* Although written for Unity/C#, the concepts in the [Unity Entities Package Documentation](https://docs.unity3d.com/Packages/com.unity.entities@0.14/manual/ecs_core.html) are very useful to understand.
* For technical performance details, see Mike Acton's [GDC talk](https://www.youtube.com/watch?v=p65Yt20pw0g)

### Current Limitations

* atom-atom interactions are not implemented. Most of our work deals with atom sources, which have low steady-state number densities, so we haven't implemented this. Results for steady-state 3D MOTs should be interpreted carefully.

## Credits

* [Xuhui Chen](https://github.com/Pi-sun), Oxford

* [Elliot Bentine](https://github.com/ElliotB256), Oxford

* [Maurice Zeuner](https://github.com/MauriceZeuner), Cambridge

* [Tiffany Harte](https://github.com/tiffanyharte), Cambridge
