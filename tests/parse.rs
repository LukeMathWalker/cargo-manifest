extern crate cargo_toml;
extern crate toml;
use cargo_toml::TomlManifest;
use std::fs::read;

#[test]
fn own() {
    let m = TomlManifest::from_slice(&read("Cargo.toml").unwrap()).unwrap();
    assert_eq!("cargo_toml", m.package.name);
    let m = TomlManifest::<toml::Value>::from_slice_with_metadata(&read("Cargo.toml").unwrap()).unwrap();
    assert_eq!("cargo_toml", m.package.name);
}
