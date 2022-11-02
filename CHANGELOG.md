# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## Important changes

Dictionaries with boolean values are now exported correctly, see
[#33](https://github.com/MForster/factorio-exporter/issues/33)

## [0.2.0] - 2022-11-02

### New features

- Factorio Exporter can now export icon paths for prototypes.

### Incompatible changes

- To avoid double parsing, `FactorioExporter::export` now returns a
  `serde_yaml::Value` instead of a `String`.

## [0.1.2] - 2022-11-01

## [0.1.1] - 2022-11-01

### Important changes

- The MSRV is now 1.60.0. This makes only explicit what already was the
  requirement before.

### Maintenance

- Start using `cargo-release` to manage version numbers and change logs.
- Minor documentation fixes.

## 0.1.0 - 2022-11-01

### New features

- Initial release
- Added `factorio_exporter` binary that can export prototypes from Factorio.
- Added the full feature as a library that can be embedded into other apps.

<!-- next-url -->
[Unreleased]: https://github.com/MForster/factorio-export/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/MForster/factorio-export/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/MForster/factorio-export/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/MForster/factorio-exporter/compare/v0.1.0...v0.1.1
