# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Incompatible changes

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
[Unreleased]: https://github.com/MForster/factorio-rust-tools/compare/factorio-cli-v0.1.0...HEAD
[0.1.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.5.1...factorio-cli-v0.1.0
