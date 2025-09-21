use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

pub mod cargo;
pub mod git;

pub const GITIGNORE: &str = "audit_build.log";
pub const JSON_CACHE: &str = "trust_store.json";

/// Standard log file location.
pub fn log_file(repo_root: &Path) -> Result<File> {
    Ok(OpenOptions::new()
        .create(true)
        .append(true)
        .open(repo_root.join("audit_build.log"))?)
}

/// Repository for the trust cache and build.rs files.
pub fn repository() -> Result<PathBuf> {
    Ok(home::cargo_home()?.join("audits").join("build-rs"))
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BuildTrustStore(pub HashMap<String, (bool, HashSet<String>)>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub packages: Vec<PackageMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub targets: Vec<PackageTarget>,
}

impl PackageMetadata {
    pub fn build_script(&self) -> PathBuf {
        self.targets
            .iter()
            .find_map(|t| {
                if t.kind.contains(&"custom-build".to_owned()) {
                    Some(PathBuf::from(&t.src_path))
                } else {
                    None
                }
            })
            .expect("to find build script in src_path")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageTarget {
    pub kind: Vec<String>,
    pub src_path: String,
}
