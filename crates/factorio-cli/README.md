# Factorio CLI

A small utility to manipulate data related to [Factorio](http://www.factorio.com).

The main feature at this point is to export prototype definitions from Factorio
using [`factorio-exporter`](http://crates.io/crates/factorio-exporter).

See crate page on [crates.io](https://crates.io/crates/factorio-cli)

## Example invocation
<!-- EMBED: fct --factorio-dir ~/factorio export -f json | jq '.recipe_prototypes["iron-plate"]' -->
```sh
$ fct --factorio-dir ~/factorio export -f json | jq '.recipe_prototypes["iron-plate"]'
{
  "allow_as_intermediate": true,
  "allow_decomposition": true,
  "allow_inserter_overload": true,
  "allow_intermediates": true,
  "always_show_made_in": false,
  "always_show_products": false,
  "category": "smelting",
  "emissions_multiplier": 1,
  "enabled": true,
  "energy": 3.2,
  "group": {
    "name": "intermediate-products"
  },
  "hidden": false,
  "hidden_from_flow_stats": false,
  "hidden_from_player_crafting": false,
  "ingredients": [
    {
      "amount": 1,
      "name": "iron-ore",
      "type": "item"
    }
  ],
  "localised_description": "Unknown key: \"recipe-description.iron-plate\"",
  "localised_name": "Iron plate",
  "main_product": {
    "amount": 1,
    "name": "iron-plate",
    "probability": 1,
    "type": "item"
  },
  "name": "iron-plate",
  "object_name": "LuaRecipePrototype",
  "order": "b[iron-plate]",
  "overload_multiplier": 0,
  "products": [
    {
      "amount": 1,
      "name": "iron-plate",
      "probability": 1,
      "type": "item"
    }
  ],
  "request_paste_multiplier": 30,
  "show_amount_in_title": true,
  "subgroup": {
    "name": "raw-material"
  },
  "unlock_results": true,
  "valid": true
}
```
<!-- END EMBED -->

## Command line

<!-- EMBED: fct help -->
```sh
$ fct help
A collection of tools for Factorio (http://www.factorio.com)

Usage: fct [OPTIONS] <COMMAND>

Commands:
  export
          Exports prototypes from Factorio in JSON or YAML format
  resolve-mods
          Lists all dependencies of a set of mods, trying to find compatible versions
  help
          Print this message or the help of the given subcommand(s)

Options:
      --factorio-dir <FACTORIO_DIR>
          Directory where Factorio is installed. This needs to be the full version. Neither the demo nor the headless version are sufficient. This argument is optional if both of `--factorio-api-spec` and `--factorio-binary` are specified

      --factorio-api-spec <FACTORIO_API_SPEC>
          Location of the `runtime-api.json` file. Defaults to `<FACTORIO_DIR>/doc-html/runtime-api.json`.
          
          The spec can be found in the `doc-html` directory of a full Factorio installation, or [online](https://lua-api.factorio.com/latest/runtime-api.json).

      --factorio-binary <FACTORIO_BINARY>
          Location of the factorio binary. Defaults to `<FACTORIO_DIR>/bin/x64/factorio(.exe)`. This can be any Factorio binary (full, headless, demo)

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information
```
<!-- END EMBED -->

<!-- EMBED: fct help export -->
```sh
$ fct help export
Exports prototypes from Factorio in JSON or YAML format

Usage: fct export [OPTIONS] [MODS]...

Arguments:
  [MODS]...  Mods to install before exporting the prototypes

Options:
  -d, --destination <DESTINATION>  Path where the result should be written. Uses STDOUT if not specified
  -f, --format <FORMAT>            Format of the output [default: json] [possible values: json, yaml]
  -i, --icons                      Export icon paths
  -h, --help                       Print help information
```
<!-- END EMBED -->

<!-- EMBED: fct help resolve-mods -->
```sh
$ fct help resolve-mods
Lists all dependencies of a set of mods, trying to find compatible versions

Usage: fct resolve-mods [MODS]...

Arguments:
  [MODS]...  A list of mods, optionally with version requirements

Options:
  -h, --help  Print help information
```
<!-- END EMBED -->

## Status

This is still very much in the prototype phase. The output will be incomplete
and have bugs. Please try it out anyway and report any issues that you run into!

See the [change log](CHANGELOG.md) for progress.

## Platform support

This tool is intended to be platform-independent, but it's currently only
tested on Linux.

## Contributing

Contributions are welcome! Feel free to send pull requests, but if you want to
make large-scale changes it would make sense to discuss them first.
