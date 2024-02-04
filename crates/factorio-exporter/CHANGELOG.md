# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Internal cleanup

- The `factorio-exporter` now uses the `--dump-data` option of the Factorio
  binary to export the prototype definitions. This is much simpler, more
  reliable and faster than the previous method of using a generated mod.

## [0.9.0] - 2022-11-26

### Incompatible changes

- The minimum supported Rust version is now 1.65.0

### Minor changes

- The exporter now loads mods only once, speeding up the process considerably.

## [0.8.0] - 2022-11-09

## [0.6.0] - 2022-11-05

### Incompatible changes

- The `factorio-exporter` binary was split into its own crate `factorio-cli`.

## [0.5.1] - 2022-11-05

### Minor changes

- Most of the public API is now initially documented.

## [0.5.0] - 2022-11-04

### Incompatible changes

- Infinite recursion is now handled in a more principled way: The object graph
  is only cut when reaching a "top-level" class, i.e. a class that is contained
  in one of the `game.*_prototypes` tables.
- The [`table` type](https://lua-api.factorio.com/latest/Builtin-Types.html#table)
  is now supported. This leads to more information in the export.

### Minor changes

- Instead of blacklisting a few specific properties that Factorio doesn't allow
  to be read without declaring them as such in the metadata, we now protect all
  property accesses with `pcall`.

## [0.4.0] - 2022-11-03

### Incompatible changes

- More string and number values are exported correctly:

```diff
       "equipment_categories": [
-        null
+        "armor"
       ],
```

### New features

- It is now possible to specify the location of the API spec and the Factorio
  binary individually, using the new command line options `--factorio-api-spec`
  and `--factorio-binary`. If both of them are specified, no factorio directory
  needs to be specified

### Incompatible changes

- The positional command line argument for the factorio directory has been
  replaced with a new option `--factorio_dir`. It is optional if both of
  `--factorio-api-spec` and `--factorio-binary` are specified.

## [0.3.0] - 2022-11-03

### New features

- It is now possible to install mods before exporting the prototype definitions

## Incompatible changes

- Dictionaries and boolean values are now exported correctly, see
 [#33](https://github.com/MForster/factorio-rust-tools/issues/33). Example:

```diff
     "assembling-machine-2": {
       "additional_pastable_entities": null,
       "allow_copy_paste": true,
-      "allowed_effects": null,
+      "allowed_effects": {
+        "consumption": true,
+        "pollution": true,
+        "productivity": true,
+        "speed": true
+      },
       "attack_result": null,
       "build_base_evolution_requirement": 0,
       "building_grid_bit_shift": 1,
```

```diff
       "name": "advanced-electronics-2",
       "object_name": "LuaTechnologyPrototype",
       "order": "a-d-c",
-      "prerequisites": null,
+      "prerequisites": {
+        "chemical-science-pack": {
+          "name": "chemical-science-pack"
+        }
+      },
       "research_unit_count": 300,
       "research_unit_energy": 1800,
       "research_unit_ingredients": [
```

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
[Unreleased]: https://github.com/MForster/factorio-rust-tools/compare/factorio-exporter-v0.9.0...HEAD
[0.9.0]: https://github.com/MForster/factorio-rust-tools/compare/factorio-exporter-v0.8.0...factorio-exporter-v0.9.0
[0.8.0]: https://github.com/MForster/factorio-rust-tools/compare/factorio-exporter-v0.7.0...factorio-exporter-v0.8.0
[0.6.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.5.1...factorio-exporter-v0.6.0
[0.5.1]: https://github.com/MForster/factorio-rust-tools/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/MForster/factorio-rust-tools/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/MForster/factorio-rust-tools/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/MForster/factorio-rust-tools/compare/v0.1.0...v0.1.1
