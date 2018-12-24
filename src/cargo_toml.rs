//! This crate defines `struct`s that can be deserialized with Serde
//! to load and inspect `Cargo.toml` metadata.
//!
//! See `TomlManifest::from_slice`.
use toml;

#[macro_use]
extern crate serde_derive;
use serde::Deserialize;
use std::collections::BTreeMap;

pub use toml::{de::Error, Value};

pub type TomlDepsSet = BTreeMap<String, TomlDependency>;
pub type TomlPlatformDepsSet = BTreeMap<String, TomlPlatform>;
pub type TomlFeatureSet = BTreeMap<String, Vec<String>>;

/// The top-level `Cargo.toml` structure
///
/// The `Metadata` is a type for `[package.metadata]` table. You can replace it with
/// your own struct type if you use the metadata and don't want to use the catch-all `Value` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlManifest<Metadata = Value> {
    pub package: TomlPackage<Metadata>,
    #[serde(default)]
    pub dependencies: TomlDepsSet,
    #[serde(default)]
    pub dev_dependencies: TomlDepsSet,
    #[serde(default)]
    pub build_dependencies: TomlDepsSet,
    #[serde(default)]
    pub target: TomlPlatformDepsSet,
    #[serde(default)]
    pub features: TomlFeatureSet,
    /// Note that due to autobins feature this is not the complete list
    #[serde(default)]
    pub bin: Vec<TomlLibOrBin>,
    #[serde(default)]
    pub bench: Vec<TomlLibOrBin>,
    #[serde(default)]
    pub test: Vec<TomlLibOrBin>,
    #[serde(default)]
    pub example: Vec<TomlLibOrBin>,

    /// Note that due to autolibs feature this is not the complete list
    pub lib: Option<TomlLibOrBin>,
    #[serde(default)]
    pub profile: TomlProfiles,
}

fn default_true() -> bool {
    true
}

impl TomlManifest<Value> {
    /// Parse contents of a `Cargo.toml` file already loaded as a byte slice
    pub fn from_slice(cargo_toml_content: &[u8]) -> Result<Self, Error> {
        Self::from_slice_with_metadata(cargo_toml_content)
    }

    /// Parse contents of a `Cargo.toml` file loaded as a string
    ///
    /// Note: this is **not** a file name, but file's content.
    pub fn from_str(cargo_toml_content: &str) -> Result<Self, Error> {
        match toml::from_str(cargo_toml_content) {
            Ok(manifest) => Ok(manifest),
            Err(e) => Self::fudge_parse(cargo_toml_content).ok_or(e),
        }
    }
}

impl<Metadata: for<'a> Deserialize<'a>> TomlManifest<Metadata> {
    /// Parse `Cargo.toml`, and parse its `[package.metadata]` into a custom Serde-compatible type
    pub fn from_slice_with_metadata(cargo_toml_content: &[u8]) -> Result<Self, Error> {
        match toml::from_slice(cargo_toml_content) {
            Ok(manifest) => Ok(manifest),
            Err(e) => std::str::from_utf8(cargo_toml_content).ok().and_then(Self::fudge_parse).ok_or(e),
        }
    }

    /// Some old crates lack the `[package]` header
    fn fudge_parse(cargo_toml_content: &str) -> Option<Self> {
        let fudged = format!("[package]\n{}", cargo_toml_content.replace("[project]", ""));
        toml::from_str(&fudged).ok()
    }
    }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TomlProfiles {
    pub release: Option<TomlProfile>,
    pub dev: Option<TomlProfile>,
    pub test: Option<TomlProfile>,
    pub bench: Option<TomlProfile>,
    pub doc: Option<TomlProfile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlProfile {
    pub opt_level: Option<Value>,
    pub debug: Option<Value>,
    pub rpath: Option<bool>,
    pub lto: Option<Value>,
    pub debug_assertions: Option<bool>,
    pub codegen_units: Option<u16>,
    pub panic: Option<String>,
    pub incremental: Option<bool>,
    pub overflow_checks: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TomlLibOrBin {
    /// This field points at where the crate is located, relative to the `Cargo.toml`.
    pub path: Option<String>,

    /// The name of a target is the name of the library or binary that will be generated.
    /// This is defaulted to the name of the package, with any dashes replaced
    /// with underscores. (Rust `extern crate` declarations reference this name;
    /// therefore the value must be a valid Rust identifier to be usable.)
    pub name: Option<String>,

    /// A flag for enabling unit tests for this target. This is used by `cargo test`.
    pub test: Option<bool>,

    /// A flag for enabling documentation tests for this target. This is only relevant
    /// for libraries, it has no effect on other sections. This is used by
    /// `cargo test`.
    pub doctest: Option<bool>,

    /// A flag for enabling benchmarks for this target. This is used by `cargo bench`.
    pub bench: Option<bool>,

    /// A flag for enabling documentation of this target. This is used by `cargo doc`.
    pub doc: Option<bool>,

    /// If the target is meant to be a compiler plugin, this field must be set to true
    /// for Cargo to correctly compile it and make it available for all dependencies.
    pub plugin: Option<bool>,

    /// If the target is meant to be a "macros 1.1" procedural macro, this field must
    /// be set to true.
    pub proc_macro: Option<bool>,

    /// If set to false, `cargo test` will omit the `--test` flag to rustc, which
    /// stops it from generating a test harness. This is useful when the binary being
    /// built manages the test runner itself.
    pub harness: Option<bool>,

    /// If set then a target can be configured to use a different edition than the
    /// `[package]` is configured to use, perhaps only compiling a library with the
    /// 2018 edition or only compiling one unit test with the 2015 edition. By default
    /// all targets are compiled with the edition specified in `[package]`.
    #[serde(default)]
    pub edition: Option<Edition>,

    /// The required-features field specifies which features the target needs in order to be built.
    /// If any of the required features are not selected, the target will be skipped.
    /// This is only relevant for the `[[bin]]`, `[[bench]]`, `[[test]]`, and `[[example]]` sections,
    /// it has no effect on `[lib]`.
    #[serde(default)]
    pub required_features: Vec<String>,

    /// The available options are "dylib", "rlib", "staticlib", "cdylib", and "proc-macro".
    #[serde(default)]
    pub crate_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlPlatform {
    #[serde(default)]
    pub dependencies: TomlDepsSet,
    #[serde(default)]
    pub dev_dependencies: TomlDepsSet,
    #[serde(default)]
    pub build_dependencies: TomlDepsSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TomlDependency {
    Simple(String),
    Detailed(TomlDependencyDetail),
}

impl TomlDependency {
    pub fn req(&self) -> &str {
        match *self {
            TomlDependency::Simple(ref v) => v,
            TomlDependency::Detailed(ref d) => d.version.as_ref().map(|s| s.as_str()).unwrap_or("*"),
        }
    }

    pub fn req_features(&self) -> &[String] {
        match *self {
            TomlDependency::Simple(_) => &[],
            TomlDependency::Detailed(ref d) => &d.features,
        }
    }

    pub fn optional(&self) -> bool {
        match *self {
            TomlDependency::Simple(_) => false,
            TomlDependency::Detailed(ref d) => d.optional,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlDependencyDetail {
    pub version: Option<String>,
    pub registry: Option<String>,
    pub registry_index: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    pub default_features: Option<bool>,
    pub package: Option<String>,
}

/// You can replace `Metadata` type with your own
/// to parse into something more useful than a generic toml `Value`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlPackage<Metadata = Value> {
    /// Careful: some names are uppercase
    pub name: String,
    #[serde(default)]
    pub edition: Edition,
    /// e.g. "1.9.0"
    pub version: String,
    pub build: Option<Value>,
    pub workspace: Option<String>,
    #[serde(default)]
    /// e.g. ["Author <e@mail>", "etc"]
    pub authors: Vec<String>,
    pub links: Option<String>,
    /// A short blurb about the package. This is not rendered in any format when
    /// uploaded to crates.io (aka this is not markdown).
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    /// This points to a file under the package root (relative to this `Cargo.toml`).
    pub readme: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    /// This is a list of up to five categories where this crate would fit.
    /// e.g. ["command-line-utilities", "development-tools::cargo-plugins"]
    pub categories: Vec<String>,
    /// e.g. "MIT"
    pub license: Option<String>,
    #[serde(rename = "license-file")]
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub metadata: Option<Metadata>,

    #[serde(default = "default_true")]
    pub autobins: bool,
    #[serde(default = "default_true")]
    pub autoexamples: bool,
    #[serde(default = "default_true")]
    pub autotests: bool,
    #[serde(default = "default_true")]
    pub autobenches: bool,
    #[serde(default = "default_true")]
    pub publish: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Edition {
    #[serde(rename = "2015")]
    E2015,
    #[serde(rename = "2018")]
    E2018,
}

impl Default for Edition {
    fn default() -> Self {
        Edition::E2015
    }
}
