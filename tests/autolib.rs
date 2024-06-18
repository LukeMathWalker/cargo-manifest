use cargo_manifest::Manifest;

#[test]
fn test_bin() {
    let m = Manifest::from_path("tests/autolib/bin/Cargo.toml").unwrap();
    assert!(m.lib.is_none());
}

#[test]
fn test_lib_rs() {
    let m = Manifest::from_path("tests/autolib/lib_rs/Cargo.toml").unwrap();

    let lib = m.lib.unwrap();
    assert_eq!(lib.path.as_deref(), Some("src/lib.rs"));
    assert_eq!(lib.name.as_deref(), Some("auto_lib"));

    insta::assert_debug_snapshot!(lib);
}

#[test]
fn test_name_override() {
    let m = Manifest::from_path("tests/autolib/name_override/Cargo.toml").unwrap();

    let lib = m.lib.unwrap();
    assert_eq!(lib.path.as_deref(), Some("src/lib.rs"));
    assert_eq!(lib.name.as_deref(), Some("foo"));

    insta::assert_debug_snapshot!(lib);
}

#[test]
fn test_path_override() {
    let m = Manifest::from_path("tests/autolib/path_override/Cargo.toml").unwrap();

    let lib = m.lib.unwrap();
    assert_eq!(lib.path.as_deref(), Some("src/foo.rs"));
    assert_eq!(lib.name.as_deref(), Some("auto_lib"));

    insta::assert_debug_snapshot!(lib);
}

#[test]
fn test_other_override() {
    let m = Manifest::from_path("tests/autolib/other_override/Cargo.toml").unwrap();

    let lib = m.lib.unwrap();
    assert!(!lib.test);
    assert!(lib.proc_macro);

    insta::assert_debug_snapshot!(lib);
}
