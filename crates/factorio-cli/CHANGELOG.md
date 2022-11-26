# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.4.0] - 2022-11-26

### New features

- Add subcommands to login to the mod portal and download mods.

### Incompatible changes

- The minimum supported Rust version is now 1.65.0

## [0.3.0] - 2022-11-09

### Incompatible changes

- There is a new subcommand `fct resolve-mods` that lists all dependencies of a
  set of mods.
- `fct` now reads a config file from `~/.config/fct/config.<EXT>`. Various formats are
  accepted, e.g. TOML, YAML, JSON,...
- The CLI feature to export prototypes has been moved to the `fct export`
  subcommand to make room for more features.

## [0.1.0] - 2022-11-05

### New features

- Initial version of the standalone `factorio-cli` create, being split out from
  `factorio-exporter`.

Earlier changes are documented in [`factorio-explorer`'s change log](../factorio-exporter/CHANGELOG.md)

<!-- next-url -->
[Unreleased]: https://github.com/MForster/factorio-rust-tools/compare/factorio-cli-v0.4.0...HEAD
[0.4.0]: https://github.com/MForster/factorio-rust-tools/compare/factorio-cli-v0.3.0...factorio-cli-v0.4.0
[0.3.0]: https://github.com/MForster/factorio-rust-tools/compare/factorio-cli-v0.2.0...factorio-cli-v0.3.0
[0.1.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.5.1...factorio-cli-v0.1.0
