use anyhow::*;
use std::{ffi::OsString, fs::copy, path::PathBuf, process::Command};

fn main() -> Result<()> {
    let mut result_path: OsString = "./.git/hooks/pre-commit".into();
    if cfg!(windows) {
        result_path.push(".exe");
    }
    let result_path: PathBuf = result_path.into();

    if !result_path.exists() {
        let target_name = "pre-commit";
        let mut command = Command::new("cargo");
        command.args(["build", "--bin", target_name, "--release"]);
        let status = command.status()?;
        if !status.success() {
            bail!("{:?} failed with status {}", command, status);
        }

        let mut executable_file_name: String = target_name.into();
        if cfg!(windows) {
            executable_file_name.push_str(".exe");
        }

        copy(
            format!("./target/release/{}", executable_file_name),
            result_path,
        )?;
    } else {
        bail!("{:?} already exists", result_path);
    }
    Ok(())
}
