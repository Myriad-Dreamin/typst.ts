use anyhow::{bail, Context, Result};
use std::{path::Path, process::Stdio};
use tokio::process::Command;

/// Run the given command and return its stdout.
pub async fn run_capture_stdout(mut command: Command) -> Result<String> {
    let output = command
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let command = command.as_std();
        bail!(
            "failed to execute `{} {}`: exited with {}\n  full command: {:?}\n {}\n {}",
            command.get_program().to_string_lossy(),
            command
                .get_args()
                .collect::<Vec<_>>()
                .join(std::ffi::OsStr::new(" "))
                .to_string_lossy(),
            output.status,
            command,
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        )
    }
}

pub async fn wasm_pack_test(
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

    run_capture_stdout(cmd)
        .await
        .context("Running Wasm tests with wasm-pack failed")
}
