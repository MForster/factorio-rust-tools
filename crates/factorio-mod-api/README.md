# Factorio Mod API

A Rust library to access the [Factorio](http://www.factorio.com) [mod portal
API](https://wiki.factorio.com/Mod_portal_API).

## Example invocation

```rust
use factorio_mod_api::ModPortalClient;

let client = ModPortalClient::new()?;
let spec = client.get_mod_spec("Warehousing").await?;

println!("{}", spec.created_at);
```

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
