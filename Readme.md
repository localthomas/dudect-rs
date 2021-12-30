# dudect-rs

This is a Rust translation of the DudeCT [paper](https://doi.org/10.23919/DATE.2017.7927267) and [C implementation](https://github.com/oreparaz/dudect).

It is not a direct translation of the C code, but serves as a playground for testing and experimenting.
Therefore no effort is put into the public facing API of this crate.
The [dudect_bencher](https://github.com/rozbb/dudect-bencher) is an alternative relevant for testing Rust code.

## Differences

There are no major functional differences, but the code is organized slightly differently.

The Welch's t-test with Welford method is separated in the `statistics` module with a simple implementation that follows the original dudect implementation.
To integrate a function that should be tested by the DudeCT method, a new trait is provided: `MeasurementSpecimen<const N: usize>` with `N` as the parameter for the length of the data blocks that are used as input to the test function.

## Development

This project provides a `flake.nix` and a `shell.nix` file, which can be used with a flake-enabled nix tool to build binaries, enter a development shell, and run checks (formatting).
Use the flake command `nix flake show` to see what is available and run `nix flake check` before committing.

#### License

Keep in mind the license of the C implementation (2016-today Oscar Reparaz).

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSES/Apache-2.0.txt) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSES/MIT.txt) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
