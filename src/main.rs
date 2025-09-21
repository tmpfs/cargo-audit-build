//! Audit build.rs files in a dependency tree.
use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const JSON_CACHE: &str = "trust_store.json";

#[derive(Default, Debug, Serialize, Deserialize)]
struct BuildTrustStore(HashMap<String, bool>);

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
struct PackageTarget {
    kind: Vec<String>,
    src_path: String,
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
    let mut out = Vec::new();
    stdout.read_to_end(&mut out)?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run cargo metadata");
    }

    serde_json::from_slice(&out).map_err(|e| anyhow!("failed to parse metadata: {}", e))
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

fn prompt(msg: &str) -> Result<String> {
    print!("{}", msg);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    Ok(input.trim().to_string())
}

fn prompt_bool(msg: &str) -> Result<bool> {
    let res = prompt(msg)?;
    Ok(matches!(res.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn make_audit_cache() -> Result<PathBuf> {
    let audits = home::cargo_home()?.join("audits").join("build-rs");

    if !audits.exists() {
        std::fs::create_dir_all(&audits)?;

        let mut ps = Command::new("git")
            .current_dir(&audits)
            .args(["init"])
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stderr = ps.stderr.take().expect("failed to capture stderr");
        let mut err = Vec::new();
        stderr.read_to_end(&mut err)?;

        let status = ps.wait()?;
        if !status.success() {
            bail!("failed to run git init: {:?}", std::str::from_utf8(&err));
        }

        let trust_cache = audits.join(JSON_CACHE);
        std::fs::write(
            &trust_cache,
            serde_json::to_string_pretty(&BuildTrustStore::default())?,
        )?;

        add_file(&audits, &trust_cache)?;
        commit_file(&audits, &trust_cache, "Initial files.")?;
    }

    Ok(audits)
}

fn add_file(repo_root: &Path, file: &Path) -> Result<()> {
    let mut ps = Command::new("git")
        .current_dir(repo_root)
        .args(["add", file.to_string_lossy().as_ref()])
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stderr = ps.stderr.take().expect("failed to capture stderr");
    let mut err = Vec::new();
    stderr.read_to_end(&mut err)?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run git commit: {:?}", std::str::from_utf8(&err));
    }

    Ok(())
}

fn commit_file(repo_root: &Path, file: &Path, msg: &str) -> Result<()> {
    let mut ps = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", msg, file.to_string_lossy().as_ref()])
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stderr = ps.stderr.take().expect("failed to capture stderr");
    let mut err = Vec::new();
    stderr.read_to_end(&mut err)?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run git commit: {:?}", std::str::from_utf8(&err));
    }

    Ok(())
}

fn is_clean(repo_root: &Path) -> Result<bool> {
    Ok(Command::new("git")
        .current_dir(repo_root)
        .args(["status", "--porcelain"])
        .output()?
        .stdout
        .is_empty())

    // git status --porcelain
}

fn commit_reviewed_trust(
    repo_root: &Path,
    build_script: &Path,
    pkg: &PackageMetadata,
) -> Result<()> {
    let cache = repo_root.join(format!("{}@{}", pkg.name, pkg.version));
    let mut input = File::open(build_script)?;
    let mut output = File::create(&cache)?;
    std::io::copy(&mut input, &mut output)?;

    let commit_msg = format!("audit-build: add build.rs for {}@{}", pkg.name, pkg.version,);
    add_file(repo_root, &cache)?;
    if !is_clean(repo_root)? {
        commit_file(repo_root, &cache, &commit_msg)?;
    }

    Ok(())
}

fn read_trust_store(repo_root: &Path) -> Result<BuildTrustStore> {
    let path = repo_root.join(JSON_CACHE);
    let file = File::open(&path)?;
    Ok(serde_json::from_reader::<_, BuildTrustStore>(file)?)
}

fn save_trust_store(repo_root: &Path, pkg_id: &str, ts: &BuildTrustStore) -> Result<()> {
    let path = repo_root.join(JSON_CACHE);
    let file = File::create(&path)?;
    serde_json::to_writer::<_, BuildTrustStore>(file, ts)?;
    let commit_msg = format!("audit-build: update trust store for {}", pkg_id);
    add_file(repo_root, &path)?;
    commit_file(repo_root, &path, &commit_msg)?;
    Ok(())
}

fn audit_build_rs(build_scripts: Vec<&PackageMetadata>, editor: &str) -> Result<()> {
    let audits = make_audit_cache()?;
    let mut trust_store = read_trust_store(&audits)?;
    for pkg in build_scripts {
        let pkg_id = format!("{}@{}", pkg.name, pkg.version);
        let is_trusted = trust_store.0.get(&pkg_id).cloned().unwrap_or(false);

        if is_trusted {
            println!("build.rs for {} is already trusted, skipping", pkg_id);
            continue;
        }

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
        } else {
            let msg = format!("do you trust the build.rs file in {}? [Y/n] ", &pkg_id);
            let trusted = prompt_bool(&msg)?;
            commit_reviewed_trust(&audits, &build_script, pkg)?;
            if !trust_store.0.contains_key(&pkg_id) || trusted != is_trusted {
                trust_store.0.insert(pkg_id.clone(), trusted);
                save_trust_store(&audits, &pkg_id, &trust_store)?;
            }
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
