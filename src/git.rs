use crate::GITIGNORE;
use anyhow::{Result, bail};
use std::path::Path;
use std::process::Command;

/// Initialize a git repository.
pub fn init_repo(repo_root: &Path) -> Result<()> {
    let log_file = crate::log_file(repo_root)?;
    let mut ps = Command::new("git")
        .current_dir(repo_root)
        .args(["init"])
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run git init");
    }

    let trust_cache = repo_root.join(crate::JSON_CACHE);
    std::fs::write(
        &trust_cache,
        serde_json::to_string_pretty(&crate::BuildTrustStore::default())?,
    )?;

    let git_ignore = repo_root.join(".gitignore");
    std::fs::write(&git_ignore, GITIGNORE)?;

    add_file(repo_root, &git_ignore)?;
    commit_file(repo_root, &git_ignore, "Initial .gitignore.")?;

    add_file(repo_root, &trust_cache)?;
    commit_file(repo_root, &trust_cache, "Initial trust store.")?;

    Ok(())
}

/// Add a file to a git repository.
pub fn add_file(repo_root: &Path, file: &Path) -> Result<()> {
    let log_file = crate::log_file(repo_root)?;
    let mut ps = Command::new("git")
        .current_dir(repo_root)
        .args(["add", file.to_string_lossy().as_ref()])
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run git add");
    }

    Ok(())
}

/// Commit a file to a git repository.
pub fn commit_file(repo_root: &Path, file: &Path, msg: &str) -> Result<()> {
    let log_file = crate::log_file(repo_root)?;

    let mut ps = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", msg, file.to_string_lossy().as_ref()])
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()?;

    let status = ps.wait()?;
    if !status.success() {
        bail!("failed to run git commit");
    }

    Ok(())
}

/// Check if a repository is clean.
pub fn is_clean(repo_root: &Path) -> Result<bool> {
    let log_file = crate::log_file(repo_root)?;
    Ok(Command::new("git")
        .current_dir(repo_root)
        .args(["status", "--porcelain"])
        .stderr(log_file)
        .output()?
        .stdout
        .is_empty())

    // git status --porcelain
}
