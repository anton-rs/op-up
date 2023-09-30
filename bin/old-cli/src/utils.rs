use std::{
    net::{SocketAddr, TcpStream},
    path::Path,
    process::{Command, Output},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use eyre::{bail, Result};
use serde_json::{Map, Value};

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

// pub fn make_executable(path: &Path) -> Result<()> {
//     let path_str = path.to_str().expect("Failed to convert path to string");
//     let out = Command::new("chmod").args(["+x", path_str]).output()?;

//     check_command(out, &format!("Failed to make {} executable", path_str))?;

//     Ok(())
// }

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

pub fn wait_up(port: u16, retries: u32, wait_secs: u64) -> Result<()> {
    for _ in 0..retries {
        println!("Trying 127.0.0.1:{}", port);
        if let Ok(stream) = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port))) {
            drop(stream);
            println!("Connected 127.0.0.1:{}", port);
            return Ok(());
        }
        thread::sleep(Duration::from_secs(wait_secs));
    }

    bail!("Timed out waiting for port {}.", port)
}

pub fn read_json(file_path: &Path) -> Result<Value> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader)?;
    Ok(json_value)
}

pub fn write_json(file_path: &Path, json_value: &Value) -> Result<()> {
    let file = std::fs::File::create(file_path)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, json_value)?;
    Ok(())
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn set_json_property<P: Into<Value>>(json_value: &mut Value, key: &str, value: P) {
    if let Some(obj) = json_value.as_object_mut() {
        obj.insert(key.to_owned(), value.into());
    } else {
        let mut obj = Map::new();
        obj.insert(key.to_owned(), value.into());
        *json_value = Value::Object(obj);
    }
}
