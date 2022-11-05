# Factorio Exporter

A Rust library to export prototype definitions from
[Factorio](http://www.factorio.com).

See crate page on [crates.io](https://crates.io/crates/factorio-exporter)

## Library

See the [module
documentation](https://docs.rs/factorio-exporter/latest/factorio_exporter/) and
the [`factorio-cli`
implementation](https://github.com/MForster/factorio-rust-tools/blob/main/crates/factorio-cli/src/main.rs)
for how to use the library.

## Status

This is still very much in the prototype phase. The output will be incomplete
and have bugs. Please try it out anyway and report any issues that you run into!

See the [change log](CHANGELOG.md) for progress.

## Design

The goal of the importer is to be as close as possible to the authoritative
definition of the prototypes. It tries to achieve that goal by two design decisions:

* The prototypes are exported from a running Factorio instance *in the runtime*
  stage. This means that the prototypes are as close as possible to how they are
  used in the game.

* The list of exported properties is taken from the [official
  definition](https://lua-api.factorio.com/latest/json-docs.html).

Another consequence of this design is that it allows to export the
prototypes of loaded mods.

## Platform support

This library is intended to be platform-independent, but it's currently only
tested on Linux.

## Contributing

Contributions are welcome! Feel free to send pull requests, but if you want to
make large-scale changes it would make sense to discuss them first.
