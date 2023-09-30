use eyre::Result;
use std::{
    net::{SocketAddr, TcpStream},
    path::Path,
    process::{Command, Output},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Clones a given git repository into the given directory.
pub fn git_clone(pwd: &Path, repo: &str) -> Result<()> {
    let out = Command::new("git")
        .arg("clone")
        .arg("--recursive")
        .arg("--depth")
        .arg("1")
        .arg(repo)
        .current_dir(pwd)
        .output()?;

    check_command(out, &format!("Failed git clone of {} in {:?}", repo, pwd))?;

    Ok(())
}
