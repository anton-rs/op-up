use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use eyre::{bail, eyre, Result};

use crate::OpStackConfig;

#[macro_export]
macro_rules! make_selection {
    ($name:ident, $prompt:expr, $options:expr) => {
        let $name = inquire::Select::new($prompt, $options)
            .without_help_message()
            .prompt()?
            .to_string();
    };
}

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

pub fn make_executable(path: &Path) -> Result<()> {
    let path_str = path.to_str().expect("Failed to convert path to string");
    let out = Command::new("chmod").args(["+x", path_str]).output()?;

    check_command(out, &format!("Failed to make {} executable", path_str))?;

    Ok(())
}

pub fn check_command(out: Output, err: &str) -> Result<()> {
    if !out.status.success() {
        bail!(
            "Failed to run command: {}: {}",
            err,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(())
}

pub fn read_stack_from_file(file: &PathBuf) -> Result<OpStackConfig> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    Ok(OpStackConfig {
        l1_client: lines
            .get(0)
            .ok_or(eyre!("expected l1_client at line 1"))?
            .to_string()
            .parse()?,
        l2_client: lines
            .get(1)
            .ok_or(eyre!("expected l2_client at line 2"))?
            .to_string()
            .parse()?,
        rollup_client: lines
            .get(2)
            .ok_or(eyre!("expected rollup_client at line 3"))?
            .to_string()
            .parse()?,
        challenger: lines
            .get(3)
            .ok_or(eyre!("expected challenger_agent at line 4"))?
            .to_string()
            .parse()?,
    })
}

pub fn write_stack_to_file(file: &PathBuf, stack: &OpStackConfig) -> Result<()> {
    let file = File::create(file)?;
    let mut writer = BufWriter::new(file);

    let mut line = String::new();
    line.push_str(&stack.l1_client.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.l2_client.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.rollup_client.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.challenger.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    Ok(())
}
