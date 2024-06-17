use cargo_manifest::Manifest;

#[test]
fn autolib() {
    let m = Manifest::from_path("tests/autolib/Cargo.toml").expect("load autolib");
    insta::assert_debug_snapshot!(m);
}
