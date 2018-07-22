//! This crate defines `struct`s that can be deserialized with Serde
//! to load and inspect `Cargo.toml` metadata.
//!
//! See `TomlManifest::from_slice`.
extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;
use serde::Deserialize;
use std::collections::BTreeMap;

pub use toml::Value;
pub use toml::de::Error;

pub type TomlDepsSet = BTreeMap<String, TomlDependency>;
pub type TomlPlatformDepsSet = BTreeMap<String, TomlPlatform>;
pub type TomlFeatureSet = BTreeMap<String, Vec<String>>;

/// The top-level `Cargo.toml` structure
///
/// The `Metadata` is a type for `[package.metadata]` table. You can replace it with
/// your own struct type if you use the metadata and don't want to use the catch-all `Value` type.
#[derive(Debug, Clone, Deserialize)]
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
    /// Note that due to autolibs feature this is not the complete list
    pub lib: Option<TomlLibOrBin>,
    #[serde(default)]
    pub profile: TomlProfiles,
}

impl TomlManifest<Value> {
    /// Parse contents of a `Cargo.toml` file loaded as a byte slice
    pub fn from_slice(cargo_toml_content: &[u8]) -> Result<Self, Error> {
        Self::from_slice_with_metadata(cargo_toml_content)
    }

    /// Parse contents of a `Cargo.toml` file loaded as a string
    ///
    /// Note: this is **not** a file name, but file's content.
    pub fn from_str(cargo_toml_content: &str) -> Result<Self, Error> {
        match toml::from_str(cargo_toml_content) {
            Ok(manifest) => Ok(manifest),
            Err(e) => {
                Self::fudge_parse(cargo_toml_content)
                .ok_or(e)
            },
        }
    }
}

impl<Metadata: for<'a> Deserialize<'a>> TomlManifest<Metadata> {
    /// Parse `Cargo.toml`, and parse its `[package.metadata]` into a custom Serde-compatible type
    pub fn from_slice_with_metadata(cargo_toml_content: &[u8]) -> Result<Self, Error> {
        match toml::from_slice(cargo_toml_content) {
            Ok(manifest) => Ok(manifest),
            Err(e) => {
                std::str::from_utf8(cargo_toml_content).ok()
                    .and_then(Self::fudge_parse)
                    .ok_or(e)
            },
        }
    }

    /// Some old crates lack the `[package]` header
    fn fudge_parse(cargo_toml_content: &str) -> Option<Self> {
        let fudged = format!("[package]\n{}", cargo_toml_content.replace("[project]",""));
        toml::from_str(&fudged).ok()
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TomlProfiles {
    pub release: Option<TomlProfile>,
    pub dev: Option<TomlProfile>,
    pub test: Option<TomlProfile>,
    pub bench: Option<TomlProfile>,
    pub doc: Option<TomlProfile>,
}

#[derive(Clone, Debug, Deserialize)]
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


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlLibOrBin {
    pub path: Option<String>,
    pub name: Option<String>,

    pub test: Option<bool>,
    pub doctest: Option<bool>,
    pub bench: Option<bool>,
    pub doc: Option<bool>,
    pub plugin: Option<bool>,
    pub proc_macro: Option<bool>,
    pub harness: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlPlatform {
    #[serde(default)]
    pub dependencies: TomlDepsSet,
    #[serde(default)]
    pub dev_dependencies: TomlDepsSet,
    #[serde(default)]
    pub build_dependencies: TomlDepsSet,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TomlDependency {
    Simple(String),
    Detailed(TomlDependencyDetail),
}

impl TomlDependency {
    pub fn req(&self) -> &str {
        match *self {
            TomlDependency::Simple(ref v) => v,
            TomlDependency::Detailed(ref d) => d.version.as_ref().map(|s|s.as_str()).unwrap_or("*"),
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

#[derive(Deserialize, Clone, Debug, Default)]
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
#[derive(Debug, Clone, Deserialize)]
pub struct TomlPackage<Metadata = Value> {
    /// Careful: some names are uppercase
    pub name: String,
    /// e.g. "1.9.0"
    pub version: String,
    pub build: Option<Value>,
    pub workspace: Option<String>,
    #[serde(default)]
    /// e.g. ["Author <e@mail>", "etc"]
    pub authors: Vec<String>,
    pub links: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    /// e.g. ["command-line-utilities", "development-tools::cargo-plugins"]
    pub categories: Vec<String>,
    /// e.g. "MIT"
    pub license: Option<String>,
    #[serde(rename = "license-file")]
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub metadata: Option<Metadata>,
}
