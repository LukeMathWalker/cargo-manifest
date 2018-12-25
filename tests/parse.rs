use cargo_toml::Manifest;
use std::fs::read;
use toml;

#[test]
fn own() {
    let m = Manifest::from_slice(&read("Cargo.toml").unwrap()).unwrap();
    assert_eq!("cargo_toml", m.package.name);
    let m = Manifest::<toml::Value>::from_slice_with_metadata(&read("Cargo.toml").unwrap()).unwrap();
    assert_eq!("cargo_toml", m.package.name);
    assert_eq!(cargo_toml::Edition::E2018, m.package.edition);
}

#[test]
fn opt_level() {
    let m = Manifest::from_slice(&read("tests/opt_level.toml").unwrap()).unwrap();
    assert_eq!("byteorder", m.package.name);
    assert_eq!(3, m.profile.bench.unwrap().opt_level.unwrap().as_integer().unwrap());
    assert_eq!(false, m.lib.unwrap().bench);
    assert_eq!(cargo_toml::Edition::E2015, m.package.edition);
}

#[test]
fn autobin() {
    let m = Manifest::from_path("tests/autobin/Cargo.toml").expect("load autobin");
    assert_eq!("auto-bin", m.package.name);
    assert_eq!(cargo_toml::Edition::E2018, m.package.edition);
    assert!(m.package.autobins);
    assert!(m.lib.is_none());
    assert_eq!(1, m.bin.len());
}

#[test]
fn autolib() {
    let m = Manifest::from_path("tests/autolib/Cargo.toml").expect("load autolib");
    assert_eq!("auto-lib", m.package.name);
    assert_eq!(false, m.package.publish);
    assert_eq!(cargo_toml::Edition::E2015, m.package.edition);
    assert!(m.package.autobins);
    assert!(!m.package.autoexamples);
    assert!(m.lib.is_some());
    assert_eq!(0, m.bin.len());
}
