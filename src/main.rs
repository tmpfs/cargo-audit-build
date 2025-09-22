//! Audit build.rs files in a dependency tree.
#![doc = include_str!("../README.md")]
use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use cargo_audit_build::{
    BuildTrustStore, JSON_CACHE, PackageMetadata,
    cargo::{fetch_dependencies, find_build_rs, package_metadata},
    git::{add_file, commit_file, init_repo, is_clean},
    repository,
};

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

fn commit_reviewed_build_script(
    repo_root: &Path,
    build_script: &Path,
    pkg: &PackageMetadata,
) -> Result<()> {
    let cache = repo_root.join(format!("{}@{}.rs", pkg.name, pkg.version));
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
    log::debug!("read trust store: {}", path.display());
    let file = File::open(&path)?;
    Ok(serde_json::from_reader::<_, BuildTrustStore>(file)?)
}

fn save_trust_store(repo_root: &Path, ts: &BuildTrustStore) -> Result<()> {
    let path = repo_root.join(JSON_CACHE);
    log::debug!("save trust store: {}", path.display());
    let file = File::create(&path)?;
    serde_json::to_writer::<_, BuildTrustStore>(file, ts)?;
    let commit_msg = "audit-build: update trust store";
    add_file(repo_root, &path)?;
    commit_file(repo_root, &path, commit_msg)?;
    Ok(())
}

fn audit_build_rs(
    repo_root: &Path,
    build_scripts: Vec<&PackageMetadata>,
    editor: &str,
) -> Result<(u32, BuildTrustStore)> {
    log::debug!("start audit: {}", repo_root.display());
    let mut num_changes = 0;
    if !repo_root.join(".git").exists() {
        log::debug!("init repository: {}", repo_root.display());
        init_repo(repo_root)?;
    }
    let mut trust_store = read_trust_store(repo_root)?;
    for pkg in build_scripts {
        let pkg_id = format!("{}@{}", pkg.name, pkg.version);
        let build_script = pkg.build_script();
        log::debug!("build script for {} is {}", pkg_id, build_script.display());
        let checksum = Sha256::digest(&std::fs::read(&build_script)?);
        let digest = format!("{:0x}", &checksum);

        let is_trusted = trust_store.0.get(&digest).map(|(v, _)| *v).unwrap_or(false);
        if is_trusted {
            let entry = trust_store.0.entry(digest).or_default();
            if !entry.1.contains(&pkg_id) {
                entry.1.insert(pkg_id.clone());
                num_changes += 1;
            }
            log::info!("build.rs for {} is already trusted, skipping", pkg_id);
            continue;
        }

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
            let msg = format!("Do you trust the build.rs file in {}? [Y/n] ", &pkg_id);
            let trusted = prompt_bool(&msg)?;
            commit_reviewed_build_script(repo_root, &build_script, pkg)?;

            let entry = trust_store.0.entry(digest).or_default();
            entry.0 = trusted;
            entry.1.insert(pkg_id.clone());
            if trusted != is_trusted {
                num_changes += 1;
            }
        }
    }
    Ok((num_changes, trust_store))
}

fn run() -> Result<()> {
    let Ok(editor) = std::env::var("EDITOR") else {
        bail!("the EDITOR environment variable must be set");
    };

    let repo_root = crate::repository()?;
    std::fs::create_dir_all(&repo_root)?;

    fetch_dependencies(&repo_root)?;
    let meta = package_metadata(&repo_root)?;
    let build_scripts = find_build_rs(&meta);
    let (num_changes, trust_store) = audit_build_rs(&repo_root, build_scripts, &editor)?;
    if num_changes > 0 {
        save_trust_store(&repo_root, &trust_store)?;
    }
    Ok(())
}

fn main() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();
    if let Err(e) = run() {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
