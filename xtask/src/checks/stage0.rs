//! Rust 母体可执行契约验收，覆盖普通与优化构建并限制子进程运行时间。

use crate::helpers::{resolve_exec, run_cmd};
use anyhow::{bail, Context, Result};
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

mod cases;

pub(crate) fn stage0_check() -> Result<()> {
    run_cmd(
        "cargo",
        &[
            "build",
            "--release",
            "-p",
            "lency_cli",
            "-p",
            "lency_runtime",
        ],
        false,
        &[],
        &[0],
    )?;
    let compiler = resolve_exec(Path::new("target/release/lencyc"))?;
    let directory = Path::new("target/stage0_check");
    fs::create_dir_all(directory)?;
    for optimized in [false, true] {
        for name in ["control", "values", "library", "files", "failure", "trace"] {
            let source = if name == "trace" {
                "tests/integration/safety/trace_panic.lcy".to_string()
            } else {
                format!("tests/stage0/{name}.lcy")
            };
            let stem = format!("{name}_{}", if optimized { "optimized" } else { "plain" });
            let executable = format!("{stem}{}", env::consts::EXE_SUFFIX);
            let mut build = Command::new(&compiler);
            build
                .args(["build", &source, "-o", &executable, "--out-dir"])
                .arg(directory);
            if optimized {
                build.arg("--release");
            }
            check_output(build, directory, &format!("build_{stem}"), 0, &[])?;
            let program = directory.join(executable).canonicalize()?;
            if name == "failure" {
                let caller_line = fs::read_to_string(&source)?
                    .lines()
                    .position(|line| line.contains("return worker.run("))
                    .context("missing method call in failure contract")?
                    + 1;
                let caller = format!("at main (tests/stage0/failure.lcy:{caller_line}:");
                for (code, diagnostic) in cases::FAILURES {
                    let mut command = command_with_runtime(&program)?;
                    command.arg(code);
                    check_output(
                        command,
                        directory,
                        &format!("{stem}_{code}"),
                        1,
                        &[diagnostic, "Lency stack:", "at Worker.run (", &caller],
                    )?;
                }
            } else if name == "trace" {
                check_output(
                    command_with_runtime(&program)?,
                    directory,
                    &stem,
                    1,
                    &[
                        "trace contract failure",
                        "at trace_failure (",
                        "trace_support.lcy:5:",
                        "trace_support.lcy:3:",
                        "at main (tests/integration/safety/trace_panic.lcy:4:",
                    ],
                )?;
            } else {
                let mut command = command_with_runtime(&program)?;
                if name == "files" {
                    command.arg(directory.join(format!("{stem}.txt")));
                }
                check_output(
                    command,
                    directory,
                    &stem,
                    0,
                    &[&format!("stage0 {name} ok")],
                )?;
            }
        }
    }
    for (name, source) in cases::REJECTED {
        let path = directory.join(format!("rejected_{name}.lcy"));
        fs::write(&path, source)?;
        let mut command = Command::new(&compiler);
        command.arg("check").arg(path);
        let mut expected = vec![cases::rejection_diagnostic(name)];
        if *name == "imported" {
            expected.push("invalid.lcy:2:");
        }
        check_output(
            command,
            directory,
            &format!("rejected_{name}"),
            1,
            &expected,
        )?;
    }
    println!("Stage0 executable contracts passed (plain and optimized).");
    Ok(())
}

fn command_with_runtime(program: &Path) -> Result<Command> {
    let runtime = PathBuf::from("target/release").canonicalize()?;
    let mut command = Command::new(program);
    for variable in ["PATH", "LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH"] {
        let mut paths = vec![runtime.clone()];
        if let Some(existing) = env::var_os(variable) {
            paths.extend(env::split_paths(&existing));
        }
        command.env(variable, env::join_paths(paths)?);
    }
    Ok(command)
}

fn check_output(
    mut command: Command,
    directory: &Path,
    name: &str,
    expected: i32,
    contains: &[&str],
) -> Result<()> {
    let stdout = directory.join(format!("{name}.stdout"));
    let stderr = directory.join(format!("{name}.stderr"));
    let mut child = command
        .stdin(Stdio::null())
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?)
        .spawn()
        .with_context(|| format!("cannot start {name}"))?;
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            let _ = child.wait();
            bail!("{name} timed out; see {}", stderr.display());
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = format!(
        "{}\n{}",
        fs::read_to_string(stdout)?,
        fs::read_to_string(stderr)?
    );
    if status.code() != Some(expected) {
        bail!("{name}: expected exit {expected}, got {status}\n{output}");
    }
    for fragment in contains {
        if !output.contains(fragment) {
            bail!("{name}: missing {fragment:?}\n{output}");
        }
    }
    println!("[stage0] PASS {name}");
    Ok(())
}
