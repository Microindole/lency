use crate::helpers::{detect_python, run_cmd, run_python, step};
use anyhow::Result;

pub(crate) fn check_rust() -> Result<()> {
    let python = detect_python()?;

    step("Checking code formatting", || {
        run_cmd(
            "cargo",
            &["fmt", "--all", "--", "--check"],
            false,
            &[],
            &[0],
        )
    })?;

    step("Running Clippy lints", || {
        run_cmd(
            "cargo",
            &[
                "clippy",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            false,
            &[],
            &[0],
        )
    })?;

    step("Running unit tests", || {
        run_cmd(
            "cargo",
            &["test"],
            false,
            &[("RUST_MIN_STACK", "8388608")],
            &[0],
        )
    })?;

    step(
        "Running .lcy integration tests (Rust compiler path)",
        || {
            if cfg!(windows) {
                run_cmd(
                    "pwsh",
                    &[
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        "scripts/win/run_lcy_tests.ps1",
                    ],
                    false,
                    &[],
                    &[0],
                )
            } else {
                run_cmd(
                    "bash",
                    &["scripts/linux/run_lcy_tests.sh"],
                    false,
                    &[],
                    &[0],
                )
            }
        },
    )?;

    step("Checking file sizes (rust scope)", || {
        run_python(
            &python,
            &["scripts/check_file_size.py", "--scope", "rust"],
            false,
        )
    })?;

    step("Scanning TODO/FIXME (rust scope)", || {
        run_python(
            &python,
            &["scripts/check_todos.py", "--scope", "rust"],
            false,
        )
    })?;

    step("Checking banned patterns (rust scope)", || {
        run_python(
            &python,
            &["scripts/check_banned_patterns.py", "--scope", "rust"],
            false,
        )
    })?;

    println!("\nAll Rust-side checks passed.");
    Ok(())
}
