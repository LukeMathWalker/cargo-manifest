# Deserialize `Cargo.toml`

This is a definition of fields in `Cargo.toml` files for [serde](https://serde.rs). It allows reading of `Cargo.toml` data. It's used by [crates.rs](https://crates.rs) [project](https://gitlab.com/crates.rs/crates.rs) to extract information about crates.

To get started, see `TomlManifest::from_slice`
