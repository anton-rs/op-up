use eyre::Result;
use std::path::Path;
use std::process::{Command, Output};

/// Check command checks the output of a command and returns an error if it failed.
pub(crate) fn check_command(out: Output, err: &str) -> Result<()> {
    if !out.status.success() {
        eyre::bail!(
            "Failed to run command: {}: {}",
            err,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Makes a file executable.
#[allow(dead_code)]
pub(crate) fn make_executable(path: &Path) -> Result<()> {
    let path_str = path.to_str().expect("Failed to convert path to string");
    let out = Command::new("chmod").args(["+x", path_str]).output()?;
    check_command(out, &format!("Failed to make {} executable", path_str))?;
    Ok(())
}
