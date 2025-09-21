use anyhow::{Result, anyhow, bail};
use std::io::Read;
use std::process::{Command, Stdio};

use crate::{Metadata, PackageMetadata};

/// Fetch dependencies.
pub fn fetch_dependencies() -> Result<()> {
    let mut ps = Command::new("cargo").arg("fetch").spawn()?;
    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to fetch dependencies");
    }
    Ok(())
}

/// Get package metadata.
pub fn package_metadata() -> Result<Metadata> {
    let mut ps = Command::new("cargo")
        .args(["metadata", "--format-version=1"])
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdout = ps.stdout.take().expect("failed to capture stdout");
    let mut out = Vec::new();
    stdout.read_to_end(&mut out)?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run cargo metadata");
    }

    serde_json::from_slice(&out).map_err(|e| anyhow!("failed to parse metadata: {}", e))
}

/// Find all the custom build scripts.
pub fn find_build_rs(meta: &Metadata) -> Vec<&PackageMetadata> {
    let mut out = Vec::new();
    for pkg in &meta.packages {
        if pkg
            .targets
            .iter()
            .any(|t| t.kind.contains(&"custom-build".to_owned()))
        {
            out.push(pkg);
        }
    }
    out
}
