use cargo_manifest::Manifest;

mod utils;

const BASIC_MANIFEST: &str = r#"
[package]
name = "auto-lib"
version = "0.1.0"
"#;

#[test]
fn test_bin() {
    let tempdir = utils::prepare(BASIC_MANIFEST, vec!["src/main.rs"]);
    let m = Manifest::from_path(tempdir.path().join("Cargo.toml")).unwrap();
    assert!(m.lib.is_none());
}

#[test]
fn test_lib_rs() {
    let tempdir = utils::prepare(BASIC_MANIFEST, vec!["src/lib.rs"]);
    let m = Manifest::from_path(tempdir.path().join("Cargo.toml")).unwrap();

    let lib = m.lib.unwrap();
    assert_eq!(lib.path.as_deref(), Some("src/lib.rs"));
    assert_eq!(lib.name.as_deref(), Some("auto_lib"));

    insta::assert_debug_snapshot!(lib);
}

#[test]
fn test_name_override() {
    let manifest = r#"
    [package]
    name = "auto-lib"
    version = "0.1.0"

    [lib]
    name = "foo"
    "#;
    let tempdir = utils::prepare(manifest, vec!["src/lib.rs"]);
    let m = Manifest::from_path(tempdir.path().join("Cargo.toml")).unwrap();

    let lib = m.lib.unwrap();
    assert_eq!(lib.path.as_deref(), Some("src/lib.rs"));
    assert_eq!(lib.name.as_deref(), Some("foo"));

    insta::assert_debug_snapshot!(lib);
}

#[test]
fn test_path_override() {
    let manifest = r#"
    [package]
    name = "auto-lib"
    version = "0.1.0"

    [lib]
    path = "src/foo.rs"
    "#;
    let tempdir = utils::prepare(manifest, vec!["src/foo.rs", "src/lib.rs"]);
    let m = Manifest::from_path(tempdir.path().join("Cargo.toml")).unwrap();

    let lib = m.lib.unwrap();
    assert_eq!(lib.path.as_deref(), Some("src/foo.rs"));
    assert_eq!(lib.name.as_deref(), Some("auto_lib"));

    insta::assert_debug_snapshot!(lib);
}

#[test]
fn test_other_override() {
    let manifest = r#"
    [package]
    name = "auto-lib"
    version = "0.1.0"
    edition = "2018"

    [lib]
    proc-macro = true
    test = false
    "#;
    let tempdir = utils::prepare(manifest, vec!["src/lib.rs"]);
    let m = Manifest::from_path(tempdir.path().join("Cargo.toml")).unwrap();

    let lib = m.lib.unwrap();
    assert!(!lib.test);
    assert!(lib.proc_macro);

    insta::assert_debug_snapshot!(lib);
}
