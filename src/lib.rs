#![allow(clippy::large_enum_variant)]
//! This crate defines `struct`s that can be deserialized with Serde
//! to load and inspect `Cargo.toml` metadata.
//!
//! See `Manifest::from_slice`.
use serde::Deserializer;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

pub use toml::Value;

pub type DepsSet = BTreeMap<String, Dependency>;
pub type TargetDepsSet = BTreeMap<String, Target>;
pub type FeatureSet = BTreeMap<String, Vec<String>>;
pub type PatchSet = BTreeMap<String, DepsSet>;

mod afs;
mod error;
pub use crate::afs::*;
pub use crate::error::Error;
use serde::de::{Error as _, Unexpected};
use std::str::FromStr;

/// The top-level `Cargo.toml` structure
///
/// The `Metadata` is a type for `[package.metadata]` table. You can replace it with
/// your own struct type if you use the metadata and don't want to use the catch-all `Value` type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest<Metadata = Value> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<Package<Metadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DepsSet>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "dev_dependencies")]
    pub dev_dependencies: Option<DepsSet>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "build_dependencies")]
    pub build_dependencies: Option<DepsSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetDepsSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<FeatureSet>,
    /// Note that due to autobins feature this is not the complete list
    /// unless you run `complete_from_path`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bin: Vec<Product>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bench: Vec<Product>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test: Vec<Product>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub example: Vec<Product>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<PatchSet>,

    /// Note that due to autolibs feature this is not the complete list
    /// unless you run `complete_from_path`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<Product>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profiles>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<Badges>,
}

impl<Metadata> Default for Manifest<Metadata> {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            package: None,
            cargo_features: None,
            workspace: None,
            dependencies: None,
            dev_dependencies: None,
            build_dependencies: None,
            target: None,
            features: None,
            patch: None,
            lib: None,
            profile: None,
            badges: None,
            bin: Default::default(),
            bench: Default::default(),
            test: Default::default(),
            example: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Workspace {
    #[serde(default)]
    pub members: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none", alias = "default_members")]
    pub default_members: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<Resolver>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DepsSet>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<WorkspacePackage>,
}

/// The workspace.package table is where you define keys that can be inherited by members of a
/// workspace. These keys can be inherited by defining them in the member package with
/// `{key}.workspace = true`.
///
/// See https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-package-table
/// for more details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct WorkspacePackage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<Edition>,
    /// e.g. "1.9.0"
    pub version: Option<String>,
    /// e.g. ["Author <e@mail>", "etc"]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A short blurb about the package. This is not rendered in any format when
    /// uploaded to crates.io (aka this is not markdown).
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// This points to a file under the package root (relative to this `Cargo.toml`).
    pub readme: Option<StringOrBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// This is a list of up to five categories where this crate would fit.
    /// e.g. ["command-line-utilities", "development-tools::cargo-plugins"]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// e.g. "MIT"
    pub license: Option<String>,
    #[serde(rename = "license-file")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<Publish>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// e.g. "1.63.0"
    #[serde(rename = "rust-version")]
    pub rust_version: Option<String>,
}

fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

impl Manifest<Value> {
    /// Parse contents of a `Cargo.toml` file already loaded as a byte slice.
    ///
    /// It does not call `complete_from_path`, so may be missing implicit data.
    pub fn from_slice(cargo_toml_content: &[u8]) -> Result<Self, Error> {
        Self::from_slice_with_metadata(cargo_toml_content)
    }

    /// Parse contents from a `Cargo.toml` file on disk.
    ///
    /// Calls `complete_from_path`.
    pub fn from_path(cargo_toml_path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::from_path_with_metadata(cargo_toml_path)
    }
}

impl FromStr for Manifest<Value> {
    type Err = Error;

    /// Parse contents of a `Cargo.toml` file loaded as a string
    ///
    /// Note: this is **not** a file name, but file's content. See `from_path`.
    ///
    /// It does not call `complete_from_path`, so may be missing implicit data.
    fn from_str(cargo_toml_content: &str) -> Result<Self, Self::Err> {
        Self::from_slice_with_metadata(cargo_toml_content.as_bytes())
    }
}

impl<Metadata: for<'a> Deserialize<'a>> Manifest<Metadata> {
    /// Parse `Cargo.toml`, and parse its `[package.metadata]` into a custom Serde-compatible type.package
    ///
    /// It does not call `complete_from_path`, so may be missing implicit data.
    pub fn from_slice_with_metadata(cargo_toml_content: &[u8]) -> Result<Self, Error> {
        let mut manifest: Self = toml_from_slice(cargo_toml_content)?;
        if manifest.package.is_none() && manifest.workspace.is_none() {
            // Some old crates lack the `[package]` header

            let val: Value = toml_from_slice(cargo_toml_content)?;
            if let Some(project) = val.get("project") {
                manifest.package = Some(project.clone().try_into()?);
            } else {
                manifest.package = Some(val.try_into()?);
            }
        }
        Ok(manifest)
    }

    /// Parse contents from `Cargo.toml` file on disk, with custom Serde-compatible metadata type.
    ///
    /// Calls `complete_from_path`
    pub fn from_path_with_metadata(cargo_toml_path: impl AsRef<Path>) -> Result<Self, Error> {
        let cargo_toml_path = cargo_toml_path.as_ref();
        let cargo_toml_content = fs::read(cargo_toml_path)?;
        let mut manifest = Self::from_slice_with_metadata(&cargo_toml_content)?;
        manifest.complete_from_path(cargo_toml_path)?;
        Ok(manifest)
    }

    /// `Cargo.toml` may not contain explicit information about `[lib]`, `[[bin]]` and
    /// `[package].build`, which are inferred based on files on disk.
    ///
    /// This scans the disk to make the data in the manifest as complete as possible.
    pub fn complete_from_path(&mut self, path: &Path) -> Result<(), Error> {
        let manifest_dir = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "bad path"))?;
        self.complete_from_abstract_filesystem(&Filesystem::new(manifest_dir))
    }

    /// `Cargo.toml` may not contain explicit information about `[lib]`, `[[bin]]` and
    /// `[package].build`, which are inferred based on files on disk.
    ///
    /// You can provide any implementation of directory scan, which doesn't have to
    /// be reading straight from disk (might scan a tarball or a git repo, for example).
    pub fn complete_from_abstract_filesystem<FS: AbstractFilesystem>(
        &mut self,
        fs: &FS,
    ) -> Result<(), Error> {
        if let Some(ref mut package) = self.package {
            let src = match fs.file_names_in("src") {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Default::default()),
                result => result,
            }?;

            let edition = match package.edition {
                Some(MaybeInherited::Local(edition)) => Some(edition),
                _ => None,
            };

            if let Some(ref mut lib) = self.lib {
                // Use `lib.name` if it's set, otherwise use `package.name` with dashes replaced by underscores
                // (see <https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field>).
                if lib.name.is_none() {
                    lib.name = Some(package.name.replace('-', "_"));
                }

                // Use `lib.path` if it's set, otherwise use `src/lib.rs` if it exists, otherwise return an error
                // (see https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-path-field).
                if lib.path.is_none() {
                    if src.contains("lib.rs") {
                        lib.path = Some("src/lib.rs".to_string());
                    } else {
                        let msg =
                            "can't find library, rename file to `src/lib.rs` or specify lib.path";
                        return Err(Error::Other(msg.to_string()));
                    }
                }

                // Use `lib.edition` if it's set, otherwise use `package.edition` unless it's inherited
                // (see https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-edition-field).
                if lib.edition.is_none() {
                    lib.edition = edition;
                }

                // Use `lib.crate_type` if it's set, otherwise use `["lib"]`
                // (see https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field).
                if lib.crate_type.is_none() {
                    lib.crate_type = Some(vec!["lib".to_string()]);
                }

                // `lib.required-features` has no effect on `[lib]`
                // (see https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-required-features-field).
                lib.required_features.clear();
            } else if src.contains("lib.rs") {
                self.lib = Some(Product {
                    name: Some(package.name.replace('-', "_")),
                    path: Some("src/lib.rs".to_string()),
                    edition,
                    crate_type: Some(vec!["lib".to_string()]),
                    ..Product::default()
                })
            }

            if matches!(package.autobins, None | Some(true)) && self.bin.is_empty() {
                let mut bin = autoset(package, "src/bin", fs);
                if src.contains("main.rs") {
                    bin.push(Product {
                        name: Some(package.name.clone()),
                        path: Some("src/main.rs".to_string()),
                        edition,
                        crate_type: Some(vec!["bin".to_string()]),
                        ..Product::default()
                    })
                }
                self.bin = bin;
            }
            if matches!(package.autoexamples, None | Some(true)) && self.example.is_empty() {
                self.example = autoset(package, "examples", fs);
            }
            if matches!(package.autotests, None | Some(true)) && self.test.is_empty() {
                self.test = autoset(package, "tests", fs);
            }
            if matches!(package.autobenches, None | Some(true)) && self.bench.is_empty() {
                self.bench = autoset(package, "benches", fs);
            }

            if matches!(package.build, None | Some(StringOrBool::Bool(true)))
                && fs.file_names_in(".")?.contains("build.rs")
            {
                package.build = Some(StringOrBool::String("build.rs".to_string()));
            }
        }
        Ok(())
    }
}

fn autoset<T, FS: AbstractFilesystem>(package: &Package<T>, dir: &str, fs: &FS) -> Vec<Product> {
    let mut out = Vec::new();
    let edition = match package.edition {
        Some(MaybeInherited::Local(edition)) => Some(edition),
        _ => None,
    };
    if let Ok(bins) = fs.file_names_in(dir) {
        for name in bins {
            let rel_path = format!("{}/{}", dir, name);
            if name.ends_with(".rs") {
                out.push(Product {
                    name: Some(name.trim_end_matches(".rs").into()),
                    path: Some(rel_path),
                    edition,
                    crate_type: Some(vec!["bin".to_string()]),
                    ..Product::default()
                })
            } else if let Ok(sub) = fs.file_names_in(&rel_path) {
                if sub.contains("main.rs") {
                    out.push(Product {
                        name: Some(name.into()),
                        path: Some(rel_path + "/main.rs"),
                        edition,
                        crate_type: Some(vec!["bin".to_string()]),
                        ..Product::default()
                    })
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Profiles {
    pub release: Option<Profile>,
    pub dev: Option<Profile>,
    pub test: Option<Profile>,
    pub bench: Option<Profile>,
    pub doc: Option<Profile>,

    #[serde(flatten)]
    pub custom: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "toml::Value")]
pub enum StripSetting {
    /// false
    None,
    Debuginfo,
    /// true
    Symbols,
}

impl Serialize for StripSetting {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::None => serializer.serialize_bool(false),
            Self::Debuginfo => serializer.serialize_str("debuginfo"),
            Self::Symbols => serializer.serialize_bool(true),
        }
    }
}

impl TryFrom<Value> for StripSetting {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Error> {
        Ok(match v {
            Value::Boolean(b) => {
                if b {
                    Self::Symbols
                } else {
                    Self::None
                }
            }
            Value::String(s) => match s.as_str() {
                "none" => Self::None,
                "debuginfo" => Self::Debuginfo,
                "symbols" => Self::Symbols,
                other => {
                    return Err(Error::Other(format!(
                        "'{other}' is not a valid value for 'strip'"
                    )))
                }
            },
            _ => {
                return Err(Error::Other(
                    "wrong data type for strip setting".to_string(),
                ))
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    #[serde(alias = "opt_level")]
    pub opt_level: Option<Value>,
    pub debug: Option<Value>,
    pub rpath: Option<bool>,
    pub inherits: Option<String>,
    pub lto: Option<Value>,
    #[serde(alias = "debug_assertions")]
    pub debug_assertions: Option<bool>,
    #[serde(alias = "codegen_units")]
    pub codegen_units: Option<u16>,
    pub panic: Option<String>,
    pub incremental: Option<bool>,
    #[serde(alias = "overflow_checks")]
    pub overflow_checks: Option<bool>,
    pub strip: Option<StripSetting>,
    #[serde(default)]
    pub package: BTreeMap<String, Value>,
    pub split_debuginfo: Option<String>,
    /// profile overrides
    pub build_override: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Cargo uses the term "target" for both "target platform" and "build target" (the thing to build),
/// which makes it ambigous.
/// Here Cargo's bin/lib **target** is renamed to **product**.
pub struct Product {
    /// This field points at where the crate is located, relative to the `Cargo.toml`.
    pub path: Option<String>,

    /// The name of a product is the name of the library or binary that will be generated.
    /// This is defaulted to the name of the package, with any dashes replaced
    /// with underscores. (Rust `extern crate` declarations reference this name;
    /// therefore the value must be a valid Rust identifier to be usable.)
    pub name: Option<String>,

    /// A flag for enabling unit tests for this product. This is used by `cargo test`.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub test: bool,

    /// A flag for enabling documentation tests for this product. This is only relevant
    /// for libraries, it has no effect on other sections. This is used by
    /// `cargo test`.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub doctest: bool,

    /// A flag for enabling benchmarks for this product. This is used by `cargo bench`.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub bench: bool,

    /// A flag for enabling documentation of this product. This is used by `cargo doc`.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub doc: bool,

    /// If the product is meant to be a compiler plugin, this field must be set to true
    /// for Cargo to correctly compile it and make it available for all dependencies.
    #[serde(default)]
    pub plugin: bool,

    /// If the product is meant to be a "macros 1.1" procedural macro, this field must
    /// be set to true.
    #[serde(default, alias = "proc_macro")]
    pub proc_macro: bool,

    /// If set to false, `cargo test` will omit the `--test` flag to rustc, which
    /// stops it from generating a test harness. This is useful when the binary being
    /// built manages the test runner itself.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub harness: bool,

    /// If set then a product can be configured to use a different edition than the
    /// `[package]` is configured to use, perhaps only compiling a library with the
    /// 2018 edition or only compiling one unit test with the 2015 edition. By default
    /// all products are compiled with the edition specified in `[package]`.
    #[serde(default)]
    pub edition: Option<Edition>,

    /// The required-features field specifies which features the product needs in order to be built.
    /// If any of the required features are not selected, the product will be skipped.
    /// This is only relevant for the `[[bin]]`, `[[bench]]`, `[[test]]`, and `[[example]]` sections,
    /// it has no effect on `[lib]`.
    #[serde(default, alias = "required_features")]
    pub required_features: Vec<String>,

    /// The available options are "dylib", "rlib", "staticlib", "cdylib", and "proc-macro".
    #[serde(skip_serializing_if = "Option::is_none", alias = "crate_type")]
    pub crate_type: Option<Vec<String>>,
}

impl Default for Product {
    fn default() -> Self {
        Self {
            path: None,
            name: None,
            test: true,
            doctest: true,
            bench: true,
            doc: true,
            harness: true,
            plugin: false,
            proc_macro: false,
            required_features: Vec::new(),
            crate_type: None,
            edition: Some(Edition::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Target {
    #[serde(default)]
    pub dependencies: DepsSet,
    #[serde(default, alias = "dev_dependencies")]
    pub dev_dependencies: DepsSet,
    #[serde(default, alias = "build_dependencies")]
    pub build_dependencies: DepsSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Inherited(InheritedDependencyDetail), // order is important for serde
    Detailed(DependencyDetail),
}

impl Dependency {
    pub fn detail(&self) -> Option<&DependencyDetail> {
        match *self {
            Dependency::Detailed(ref d) => Some(d),
            _ => None,
        }
    }

    /// Simplifies `Dependency::Detailed` to `Dependency::Simple` if only the
    /// `version` field inside the `DependencyDetail` struct is set.
    pub fn simplify(self) -> Self {
        match self {
            Dependency::Detailed(details) => details.simplify(),
            dep => dep,
        }
    }

    pub fn req(&self) -> &str {
        match *self {
            Dependency::Simple(ref v) => v,
            Dependency::Detailed(ref d) => d.version.as_deref().unwrap_or("*"),
            Dependency::Inherited(_) => "*",
        }
    }

    pub fn req_features(&self) -> &[String] {
        match *self {
            Dependency::Simple(_) => &[],
            Dependency::Detailed(ref d) => d.features.as_deref().unwrap_or(&[]),
            Dependency::Inherited(ref i) => i.features.as_deref().unwrap_or(&[]),
        }
    }

    pub fn optional(&self) -> bool {
        match *self {
            Dependency::Simple(_) => false,
            Dependency::Detailed(ref d) => d.optional.unwrap_or(false),
            Dependency::Inherited(ref i) => i.optional.unwrap_or(false),
        }
    }

    // `Some` if it overrides the package name.
    // If `None`, use the dependency name as the package name.
    pub fn package(&self) -> Option<&str> {
        match *self {
            Dependency::Detailed(ref d) => d.package.as_deref(),
            _ => None,
        }
    }

    // Git URL of this dependency, if any
    pub fn git(&self) -> Option<&str> {
        self.detail().and_then(|d| d.git.as_deref())
    }

    // `true` if it's an usual crates.io dependency,
    // `false` if git/path/alternative registry
    pub fn is_crates_io(&self) -> bool {
        match *self {
            Dependency::Simple(_) => true,
            Dependency::Detailed(ref d) => {
                // TODO: allow registry to be set to crates.io explicitly?
                d.path.is_none()
                    && d.registry.is_none()
                    && d.registry_index.is_none()
                    && d.git.is_none()
                    && d.tag.is_none()
                    && d.branch.is_none()
                    && d.rev.is_none()
            }
            Dependency::Inherited(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(alias = "registry_index")]
    pub registry_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(default, alias = "default_features")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_features: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

impl DependencyDetail {
    fn simplify(self) -> Dependency {
        let Self {
            version: _,
            registry,
            registry_index,
            path,
            git,
            branch,
            tag,
            rev,
            features,
            optional,
            default_features,
            package,
        } = &self;

        if registry.is_some()
            || registry_index.is_some()
            || path.is_some()
            || git.is_some()
            || branch.is_some()
            || tag.is_some()
            || rev.is_some()
            || features.is_some()
            || optional.is_some()
            || default_features.is_some()
            || package.is_some()
        {
            return Dependency::Detailed(self);
        }

        match self.version {
            None => Dependency::Detailed(self),
            Some(version) => Dependency::Simple(version),
        }
    }
}

/// When a dependency is defined as `{ workspace = true }`,
/// and workspace data hasn't been applied yet.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InheritedDependencyDetail {
    pub workspace: True,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
}

/// Used as a wrapper for properties that may be inherited by workspace-level settings.
/// It currently does not support more complex interactions (e.g. specifying part of the property
/// in the local manifest while inheriting another part of it from the workspace manifest, as it
/// happens for dependency features).
///
/// See [`cargo`'s documentation](https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#workspaces)
/// for more details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
#[serde(untagged)]
pub enum MaybeInherited<T> {
    Inherited { workspace: True },
    Local(T),
}

impl<T> MaybeInherited<T> {
    pub fn inherited() -> Self {
        Self::Inherited { workspace: True }
    }

    pub fn as_local(self) -> Option<T> {
        match self {
            Self::Local(x) => Some(x),
            Self::Inherited { .. } => None,
        }
    }

    pub const fn as_ref(&self) -> MaybeInherited<&T> {
        match self {
            Self::Local(ref x) => MaybeInherited::Local(x),
            Self::Inherited { .. } => MaybeInherited::Inherited { workspace: True },
        }
    }
}

/// A type-level representation of a `true` boolean value.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct True;

impl Serialize for True {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(true)
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if bool::deserialize(deserializer)? {
            Ok(Self)
        } else {
            Err(D::Error::invalid_value(
                Unexpected::Bool(false),
                &"a `true` boolean value",
            ))
        }
    }
}

/// You can replace `Metadata` type with your own
/// to parse into something more useful than a generic toml `Value`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package<Metadata = Value> {
    /// Careful: some names are uppercase
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<MaybeInherited<Edition>>,
    /// e.g. "1.9.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<StringOrBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// e.g. ["Author <e@mail>", "etc"]
    pub authors: Option<MaybeInherited<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A short blurb about the package. This is not rendered in any format when
    /// uploaded to crates.io (aka this is not markdown).
    pub description: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// This points to a file under the package root (relative to this `Cargo.toml`).
    pub readme: Option<MaybeInherited<StringOrBool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<MaybeInherited<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// This is a list of up to five categories where this crate would fit.
    /// e.g. ["command-line-utilities", "development-tools::cargo-plugins"]
    pub categories: Option<MaybeInherited<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// e.g. "MIT"
    pub license: Option<MaybeInherited<String>>,
    #[serde(rename = "license-file")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_file: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// e.g. "1.63.0"
    #[serde(rename = "rust-version")]
    pub rust_version: Option<MaybeInherited<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<MaybeInherited<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<MaybeInherited<Vec<String>>>,

    #[serde(rename = "default-run")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The default binary to run by cargo run.
    pub default_run: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autobins: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoexamples: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autotests: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autobenches: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<MaybeInherited<Publish>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<Resolver>,
}

impl<Metadata> Package<Metadata> {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            edition: None,
            version: Some(MaybeInherited::Local(version)),
            build: None,
            workspace: None,
            authors: None,
            links: None,
            description: None,
            homepage: None,
            documentation: None,
            readme: None,
            keywords: None,
            categories: None,
            license: None,
            license_file: None,
            repository: None,
            metadata: None,
            rust_version: None,
            exclude: None,
            include: None,
            default_run: None,
            autobins: None,
            autoexamples: None,
            autotests: None,
            autobenches: None,
            publish: None,
            resolver: None,
        }
    }

    /// Returns whether to use the legacy behavior for target auto-discovery
    /// from the 2015 Rust edition.
    ///
    /// The default value for target auto-discovery changed in the 2018 edition
    /// (see https://doc.rust-lang.org/cargo/reference/cargo-targets.html#target-auto-discovery).
    ///
    /// - If the edition is not set or is set to 2015, we use the legacy behavior.
    /// - If the edition is set to 2018 or later, we use the new behavior.
    /// - If the edition is inherited, we assume that the edition is 2018
    ///   or later, since inheritance is a newer feature.
    fn uses_legacy_auto_discovery(&self) -> bool {
        matches!(
            self.edition,
            None | Some(MaybeInherited::Local(Edition::E2015))
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum StringOrBool {
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Publish {
    Flag(bool),
    Registry(Vec<String>),
}

impl Default for Publish {
    fn default() -> Self {
        Publish::Flag(true)
    }
}

impl PartialEq<Publish> for bool {
    fn eq(&self, p: &Publish) -> bool {
        match *p {
            Publish::Flag(flag) => flag == *self,
            Publish::Registry(ref reg) => reg.is_empty() != *self,
        }
    }
}

impl PartialEq<bool> for Publish {
    fn eq(&self, b: &bool) -> bool {
        b.eq(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Badge {
    pub repository: String,
    #[serde(default = "default_master")]
    pub branch: String,
    pub service: Option<String>,
    pub id: Option<String>,
    #[serde(alias = "project_name")]
    pub project_name: Option<String>,
}

fn default_master() -> String {
    "master".to_string()
}

#[allow(clippy::unnecessary_wraps)]
fn ok_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    Ok(Deserialize::deserialize(deserializer).unwrap_or_default())
}

fn toml_from_slice<T>(s: &'_ [u8]) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    Ok(toml::from_str(std::str::from_utf8(s)?)?)
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Badges {
    /// Appveyor: `repository` is required. `branch` is optional; default is `master`
    /// `service` is optional; valid values are `github` (default), `bitbucket`, and
    /// `gitlab`; `id` is optional; you can specify the appveyor project id if you
    /// want to use that instead. `project_name` is optional; use when the repository
    /// name differs from the appveyor project name.
    #[serde(default, deserialize_with = "ok_or_default")]
    pub appveyor: Option<Badge>,

    /// Circle CI: `repository` is required. `branch` is optional; default is `master`
    #[serde(default, deserialize_with = "ok_or_default")]
    pub circle_ci: Option<Badge>,

    /// GitLab: `repository` is required. `branch` is optional; default is `master`
    #[serde(default, deserialize_with = "ok_or_default")]
    pub gitlab: Option<Badge>,

    /// Travis CI: `repository` in format "<user>/<project>" is required.
    /// `branch` is optional; default is `master`
    #[serde(default, deserialize_with = "ok_or_default")]
    pub travis_ci: Option<Badge>,

    /// Codecov: `repository` is required. `branch` is optional; default is `master`
    /// `service` is optional; valid values are `github` (default), `bitbucket`, and
    /// `gitlab`.
    #[serde(default, deserialize_with = "ok_or_default")]
    pub codecov: Option<Badge>,

    /// Coveralls: `repository` is required. `branch` is optional; default is `master`
    /// `service` is optional; valid values are `github` (default) and `bitbucket`.
    #[serde(default, deserialize_with = "ok_or_default")]
    pub coveralls: Option<Badge>,

    /// Is it maintained resolution time: `repository` is required.
    #[serde(default, deserialize_with = "ok_or_default")]
    pub is_it_maintained_issue_resolution: Option<Badge>,

    /// Is it maintained percentage of open issues: `repository` is required.
    #[serde(default, deserialize_with = "ok_or_default")]
    pub is_it_maintained_open_issues: Option<Badge>,

    /// Maintenance: `status` is required. Available options are `actively-developed`,
    /// `passively-maintained`, `as-is`, `experimental`, `looking-for-maintainer`,
    /// `deprecated`, and the default `none`, which displays no badge on crates.io.
    #[serde(default, deserialize_with = "ok_or_default")]
    pub maintenance: Maintenance,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Maintenance {
    pub status: MaintenanceStatus,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum MaintenanceStatus {
    #[default]
    None,
    ActivelyDeveloped,
    PassivelyMaintained,
    AsIs,
    Experimental,
    LookingForMaintainer,
    Deprecated,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Default)]
pub enum Edition {
    #[serde(rename = "2015")]
    #[default]
    E2015,
    #[serde(rename = "2018")]
    E2018,
    #[serde(rename = "2021")]
    E2021,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum Resolver {
    #[serde(rename = "1")]
    V1,
    #[serde(rename = "2")]
    V2,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::V1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_auto_discovery_flag() {
        let mut package = Package::<()>::new("foo".into(), "1.0.0".into());
        assert!(package.uses_legacy_auto_discovery());

        package.edition = Some(MaybeInherited::Local(Edition::E2015));
        assert!(package.uses_legacy_auto_discovery());

        package.edition = Some(MaybeInherited::Local(Edition::E2018));
        assert!(!package.uses_legacy_auto_discovery());

        package.edition = Some(MaybeInherited::Local(Edition::E2021));
        assert!(!package.uses_legacy_auto_discovery());

        package.edition = Some(MaybeInherited::Inherited { workspace: True });
        assert!(!package.uses_legacy_auto_discovery());
    }
}
