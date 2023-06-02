use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use eyre::Result;

#[macro_export]
macro_rules! make_selection {
    ($name:ident, $prompt:expr, $options:expr) => {
        let $name = inquire::Select::new($prompt, $options)
            .prompt()?
            .to_string();
    };
}

#[macro_export]
macro_rules! git_clone {
    ($options:expr, $repo:expr) => {
        std::process::Command::new("git")
            .args(["clone", $options, $repo])
            .output()
            .expect("Failed to clone repository from git");
    };
}

pub fn read_stack_from_file() -> Result<Vec<String>> {
    let file = File::open("../.stack")?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    Ok(lines)
}

pub fn write_stack_to_file(stack: Vec<String>) -> Result<()> {
    let file = File::create("../.stack")?;
    let mut writer = BufWriter::new(file);

    for mut line in stack {
        line.push('\n');
        writer.write_all(line.as_bytes())?;
    }

    Ok(())
}
