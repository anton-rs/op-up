use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use eyre::{eyre, Result};

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

#[macro_export]
macro_rules! git_clone {
    ($pwd:expr, $options:expr, $repo:expr) => {
        std::process::Command::new("git")
            .args(["clone", $options, $repo])
            .current_dir($pwd)
            .output()
            .expect(format!("Failed to clone repository {} from git", $repo).as_str());
    };
}

#[macro_export]
macro_rules! git_sparse_checkout {
    ($dir:expr, $cmd:expr, $options:expr) => {
        std::process::Command::new("git")
            .args(["sparse-checkout", $cmd, $options])
            .current_dir($dir)
            .output()
            .expect(format!("Failed to checkout {} from git", $options).as_str());
    };
}

#[macro_export]
macro_rules! make_executable {
    ($path:expr) => {
        let path_str = $path.to_str().expect("Failed to convert path to string");
        std::process::Command::new("chmod")
            .args(["+x", path_str])
            .output()
            .expect(format!("Failed to make script {:?} executable", $path).as_str());
    };
}

pub fn read_stack_from_file(file: &PathBuf) -> Result<OpStackConfig> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    Ok(OpStackConfig {
        l1_client: lines
            .get(0)
            .ok_or(eyre!("invalid l1_client found"))?
            .to_string(),
        l2_client: lines
            .get(1)
            .ok_or(eyre!("invalid l2_client found"))?
            .to_string(),
        rollup_client: lines
            .get(2)
            .ok_or(eyre!("invalid rollup_client found"))?
            .to_string(),
        challenger_agent: lines
            .get(3)
            .ok_or(eyre!("invalid challenger_agent found"))?
            .to_string(),
    })
}

pub fn write_stack_to_file(file: &PathBuf, stack: &OpStackConfig) -> Result<()> {
    let file = File::create(file)?;
    let mut writer = BufWriter::new(file);

    let mut line = String::new();
    line.push_str(&stack.l1_client);
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.l2_client);
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.rollup_client);
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.challenger_agent);
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    Ok(())
}
