# Factorio Rust Tools

This is a small collection of utilities for [Factorio](http://www.factorio.com):

* [`factorio-exporter`](crates/factorio-exporter): A Rust library to export
  prototype definitions from Factorio.
* [`factorio-cli`](crates/factorio-cli): A small utility to manipulate data
  related to Factorio, currently mostly a command-line front-end for
  `factorio-exporter`.
* [`factorio-mod-api`](crates/factorio-mod-api): A caching client library for
  the [Factorio Mod Portal API](https://wiki.factorio.com/Mod_portal_API).

## Status

This is still very much in the prototype phase. The output will be incomplete
and have bugs. Please try it out anyway and report any issues that you run into!

See the [change log](CHANGELOG.md) for progress.

## Platform support

This library is intended to be platform-independent, but it's currently only
tested on Linux.

## Contributing

Contributions are welcome! Feel free to send pull requests, but if you want to
make large-scale changes it would make sense to discuss them first.
