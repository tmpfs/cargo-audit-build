use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    packages: Vec<PackageMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageMetadata {
    name: String,
    version: String,
    targets: Vec<PackageTarget>,
    manifest_path: String,
}

impl PackageMetadata {
    fn build_script(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.manifest_path);
        path.set_file_name("build.rs");
        path
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageTarget {
    kind: Vec<String>,
}

fn fetch_dependencies() -> Result<()> {
    let mut ps = Command::new("cargo").arg("fetch").spawn()?;
    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to fetch dependencies");
    }
    Ok(())
}

fn package_metadata() -> Result<Metadata> {
    let mut ps = Command::new("cargo")
        .args(["metadata", "--format-version=1"])
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdout = ps.stdout.take().expect("failed to capture stdout");
    let mut buffer = Vec::new();
    stdout.read_to_end(&mut buffer)?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run cargo metadata");
    }

    serde_json::from_slice(&buffer).map_err(|e| anyhow!("failed to parse metadata: JSON {}", e))
}

fn find_build_rs(meta: &Metadata) -> Vec<&PackageMetadata> {
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

fn audit_build_rs(build_scripts: Vec<&PackageMetadata>, editor: &str) -> Result<()> {
    for pkg in build_scripts {
        let build_script = pkg.build_script();
        let mut ps = Command::new(editor)
            .arg(build_script.to_string_lossy().as_ref())
            .spawn()?;
        let status = ps.wait()?;
        if !status.success() {
            bail!(
                "the EDITOR ({}) exited with code {}",
                editor,
                status.code().unwrap_or(i32::MIN)
            );
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    let Ok(editor) = std::env::var("EDITOR") else {
        bail!("the EDITOR environment variable must be set");
    };
    fetch_dependencies()?;
    let meta = package_metadata()?;
    let build_scripts = find_build_rs(&meta);
    audit_build_rs(build_scripts, &editor)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
