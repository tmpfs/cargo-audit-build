use crate::{Metadata, PackageMetadata};
use anyhow::{Result, anyhow, bail};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

/// Fetch dependencies.
pub fn fetch_dependencies(repo_root: &Path) -> Result<()> {
    log::debug!("fetch dependencies");
    let log_file = crate::log_file(repo_root)?;
    let mut ps = Command::new("cargo")
        .arg("fetch")
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()?;
    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to fetch dependencies");
    }
    Ok(())
}

/// Get package metadata.
pub fn package_metadata(repo_root: &Path) -> Result<Metadata> {
    log::debug!("package metadata");
    let log_file = crate::log_file(repo_root)?;
    let mut ps = Command::new("cargo")
        .args(["metadata", "--format-version=1"])
        .stdout(Stdio::piped())
        .stderr(log_file)
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
    log::debug!("find build.rs files");
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
