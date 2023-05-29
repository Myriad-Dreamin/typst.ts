use anyhow::{bail, Context, Result};
use std::{
    path::Path,
    process::{Command, Stdio},
};

/// Run the given command and return its stdout.
pub fn run_capture_stdout(mut command: Command) -> Result<String> {
    let output = command
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(
            "failed to execute `{} {}`: exited with {}\n  full command: {:?}",
            command.get_program().to_string_lossy(),
            command
                .get_args()
                .collect::<Vec<_>>()
                .join(std::ffi::OsStr::new(" "))
                .to_string_lossy(),
            output.status,
            command,
        )
    }
}

pub fn wasm_pack_test(
    path: &Path,
    release: bool,
    features: &[&str],
    extra_options: &[&str],
) -> Result<String> {
    let mut cmd = Command::new("wasm-pack");

    cmd.current_dir(path).arg("test");

    if release {
        cmd.arg("--release");
    }

    cmd.args(extra_options);

    for feature in features {
        cmd.arg("--features").arg(feature);
    }

    run_capture_stdout(cmd).context("Running Wasm tests with wasm-pack failed")
}
