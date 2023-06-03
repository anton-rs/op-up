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

pub enum GitCloneMethod {
    Shallow,
    Full,
}

pub fn git_clone<M: Into<GitCloneMethod>>(pwd: &Path, method: M, repo: &str) -> Result<()> {
    match method.into() {
        GitCloneMethod::Full => {
            let out = Command::new("git")
                .arg("clone")
                .arg(repo)
                .current_dir(pwd)
                .output()?;

            check_command(out, &format!("Failed full clone {} in {:?}", repo, pwd))?;
        }
        GitCloneMethod::Shallow => {
            let out = Command::new("git")
                .arg("clone")
                .arg("--no-checkout")
                .arg("--filter=blob:none")
                .arg("--depth")
                .arg("1")
                .arg("--sparse")
                .arg(repo)
                .current_dir(pwd)
                .output()?;

            check_command(out, &format!("Failed shallow clone {} in {:?}", repo, pwd))?;
        }
    }

    Ok(())
}

pub fn git_sparse_checkout(dir: &Path, cmd: &str, options: &str) -> Result<()> {
    let out = Command::new("git")
        .arg("sparse-checkout")
        .arg(cmd)
        .arg(options)
        .current_dir(dir)
        .output()?;

    check_command(out, &format!("Failed sparse-checkout {} {}", cmd, options))?;

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

pub fn wait_for_response(url: &str) -> Result<()> {
    loop {
        match reqwest::blocking::get(url) {
            Ok(_) => break,
            Err(_) => {
                println!("Waiting for response from {}", url);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
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
        challenger_agent: lines
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
    line.push_str(&stack.challenger_agent.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    Ok(())
}
