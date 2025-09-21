use anyhow::{Result, bail};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

/// Initialize a git repository.
pub fn init_repo(repo_root: &Path) -> Result<()> {
    std::fs::create_dir_all(repo_root)?;

    let mut ps = Command::new("git")
        .current_dir(repo_root)
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

    let trust_cache = repo_root.join(crate::JSON_CACHE);
    std::fs::write(
        &trust_cache,
        serde_json::to_string_pretty(&crate::BuildTrustStore::default())?,
    )?;

    add_file(repo_root, &trust_cache)?;
    commit_file(repo_root, &trust_cache, "Initial files.")?;

    Ok(())
}

/// Add a file to a git repository.
pub fn add_file(repo_root: &Path, file: &Path) -> Result<()> {
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

/// Commit a file to a git repository.
pub fn commit_file(repo_root: &Path, file: &Path, msg: &str) -> Result<()> {
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

/// Check if a repository is clean.
pub fn is_clean(repo_root: &Path) -> Result<bool> {
    Ok(Command::new("git")
        .current_dir(repo_root)
        .args(["status", "--porcelain"])
        .output()?
        .stdout
        .is_empty())

    // git status --porcelain
}
