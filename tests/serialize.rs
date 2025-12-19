use cargo_manifest::{Lint, LintLevel, Manifest, MaybeInheritedLintsSet, Package};

#[test]
fn basic() {
    let manifest: Manifest<(), ()> = Manifest {
        package: Some(Package::new("foo".into(), "1.0.0".into())),
        ..Default::default()
    };

    let serialized = toml::to_string(&manifest).unwrap();
    insta::assert_snapshot!(serialized);
}

#[test]
fn with_lints() {
    let manifest: Manifest<(), ()> = Manifest {
        package: Some(Package::new("foo".into(), "1.0.0".into())),
        lints: Some(MaybeInheritedLintsSet {
            workspace: None,
            lints: [(
                "rust".into(),
                [("unused".into(), Lint::Level(LintLevel::Forbid))]
                    .into_iter()
                    .collect(),
            )]
            .into_iter()
            .collect(),
        }),
        ..Default::default()
    };

    let serialized = toml::to_string(&manifest).unwrap();
    insta::assert_snapshot!(serialized);
}
