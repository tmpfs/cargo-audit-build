use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub mod cargo;
pub mod git;

pub const JSON_CACHE: &str = "trust_store.json";

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
