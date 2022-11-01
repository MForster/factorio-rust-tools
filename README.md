# Factorio Exporter

A Rust library to export prototype definitions from
[Factorio](http://www.factorio.com).

This is both a library as well as a command line tool to export prototype
definitions from Factorio.

See crate page on [crates.io](https://crates.io/crates/factorio-exporter)

## Example invocation

```sh
$ factorio_exporter ~/tmp/factorio-full -f json -d vanilla.json
$ jq '.recipe_prototypes["iron-plate"]' vanilla.json
```
Output:

```json
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
  "unlock_results": true,
  "valid": true
}
```

## Command line

```sh
$ factorio_exporter --help
Exports prototypes from Factorio in JSON or YAML format

Usage: factorio_exporter [OPTIONS] <FACTORIO_DIR>

Arguments:
  <FACTORIO_DIR>  Directory where Factorio is installed. This needs to be the full version. Neither the demo nor the headless version are sufficient

Options:
  -d, --destination <DESTINATION>  Path where the result should be written. Uses STDOUT if not specified
  -f, --format <FORMAT>            Format of the output [default: json] [possible values: json, yaml]
  -h, --help                       Print help information
  -V, --version                    Print version information
```

## Library

See [factorio_exporter.rs](https://github.com/MForster/factorio-exporter/blob/main/src/bin/factorio_exporter.rs)
for how to use the crate as a library.

## Status

This is still very much in the prototype phase. The output will be incomplete
and have bugs. Please try it out anyway and report any issues that you run into!

## Design

The goal of the importer is to be as close as possible to the authoritative
definition of the prototypes. It tries to achieve that goal by two design decisions:

* The prototypes are exported from a running Factorio instance *in the runtime*
  stage. This means that the prototypes are as close as possible to how they are
  used in the game.

* The list of exported properties is taken from the [official
  definition](https://lua-api.factorio.com/latest/runtime-api.json).

Another consequence of this design is that it will be possible to export the
prototypes of loaded mods. This isn't implemented, yet, however.

## Contributing

Contributions are welcome! Feel free to send pull request, but if you want to
make large-scale changes it would make sense to discuss them first.
