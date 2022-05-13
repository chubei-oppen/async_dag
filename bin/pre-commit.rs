use std::process::Command;

use anyhow::*;

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        bail!("{:?} failed with status {}", command, status);
    }
    Ok(())
}

fn main() -> Result<()> {
    run(Command::new("cargo").args(["fmt", "--check"]))?;
    run(Command::new("cargo").args(["clippy", "--", "-D", "warnings"]))?;
    run(Command::new("cargo").args(["test"]))?;
    Ok(())
}
