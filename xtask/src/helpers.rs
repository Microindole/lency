use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const RESET: &str = "\x1b[0m";

fn color_enabled() -> bool {
    io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").map_or(true, |term| term != "dumb")
}

fn paint(code: &str, text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    if color_enabled() {
        format!("\x1b[{code}m{text}{RESET}")
    } else {
        text.to_string()
    }
}

pub(crate) fn print_error(error: &anyhow::Error) {
    eprintln!("{}", paint("1;31", format!("[xtask] ERROR: {error:#}")));
}

pub(crate) fn step<F>(name: &str, action: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    println!("\n{}", paint("1;36", format!("[xtask] ==> {name}")));
    match action() {
        Ok(()) => {
            println!("{}", paint("1;32", format!("[xtask] OK  {name}")));
            Ok(())
        }
        Err(error) => {
            eprintln!("{}", paint("1;31", format!("[xtask] FAIL {name}")));
            Err(error)
        }
    }
}

pub(crate) fn run_cmd<P: AsRef<Path>>(
    program: P,
    args: &[&str],
    quiet: bool,
    envs: &[(&str, &str)],
    accept_codes: &[i32],
) -> Result<()> {
    let program = program.as_ref();
    if !quiet {
        print_command_source(program, args);
    }
    let mut cmd = Command::new(program);
    prepare_command(&mut cmd, args, envs);
    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = cmd.status().with_context(|| {
        format!(
            "failed to run command: {} {}",
            program.display(),
            args.join(" ")
        )
    })?;
    let code = status.code().unwrap_or(-1);
    if !accept_codes.contains(&code) {
        bail!(
            "command failed with exit code {}: {} {}",
            code,
            program.display(),
            args.join(" ")
        );
    }
    Ok(())
}

fn print_command_source(program: &Path, args: &[&str]) {
    let name = program
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (label, color) = if name == "cargo" || name == "cargo.exe" {
        ("rust/cargo", "1;34")
    } else if name.starts_with("lencyc_") {
        ("lency/selfhost", "1;35")
    } else if name == "lencyc" || name == "lencyc.exe" {
        ("lency/rust-host", "1;35")
    } else if name.starts_with("python") || name == "py" || name == "py.exe" {
        ("tool/python", "1;33")
    } else {
        ("tool/exec", "1;33")
    };
    println!(
        "{}",
        paint(
            color,
            format!("[{label}] {} {}", program.display(), args.join(" "))
        )
    );
}

pub(crate) fn run_cmd_exit_code<P: AsRef<Path>>(
    program: P,
    args: &[&str],
    quiet: bool,
) -> Result<i32> {
    let program = program.as_ref();
    let mut cmd = Command::new(program);
    prepare_command(&mut cmd, args, &[]);
    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let status = cmd.status().with_context(|| {
        format!(
            "failed to run command: {} {}",
            program.display(),
            args.join(" ")
        )
    })?;
    Ok(status.code().unwrap_or(-1))
}

pub(crate) fn run_cmd_capture(program: &Path, args: &[&str]) -> Result<String> {
    let mut cmd = Command::new(program);
    prepare_command(&mut cmd, args, &[]);
    let output = cmd.output().with_context(|| {
        format!(
            "failed to run command: {} {}",
            program.display(),
            args.join(" ")
        )
    })?;
    if !output.status.success() {
        bail!(
            "command failed with exit code {:?}: {} {}",
            output.status.code(),
            program.display(),
            args.join(" ")
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn prepare_command(cmd: &mut Command, args: &[&str], envs: &[(&str, &str)]) {
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    inject_runtime_path(cmd);
}

fn inject_runtime_path(cmd: &mut Command) {
    if !cfg!(windows) {
        return;
    }

    let Some(runtime_dir) = find_runtime_dir() else {
        return;
    };
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![runtime_dir];
    paths.extend(std::env::split_paths(&existing));
    if let Ok(joined) = std::env::join_paths(paths) {
        cmd.env("PATH", joined);
    }
}

fn find_runtime_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let dirs = ["target/release", "target/debug"];
    let libs = [
        "lency_runtime.dll",
        "lency_runtime.dll.lib",
        "lency_runtime.lib",
        "liblency_runtime.so",
        "liblency_runtime.dylib",
        "liblency_runtime.a",
    ];

    for dir in dirs {
        let runtime_dir = cwd.join(dir);
        for lib in libs {
            if runtime_dir.join(lib).exists() {
                return Some(runtime_dir);
            }
        }
    }
    None
}

pub(crate) fn resolve_exec(base: &Path) -> Result<PathBuf> {
    let mut candidates = vec![base.to_path_buf()];
    candidates.push(PathBuf::from(format!("{}.exe", base.display())));
    for c in candidates {
        if c.exists() {
            return Ok(c);
        }
    }
    bail!("executable not found: {}(.exe)", base.display())
}

pub(crate) fn ensure_file_non_empty(path: &Path) -> Result<()> {
    let meta = fs::metadata(path).with_context(|| format!("missing file: {}", path.display()))?;
    if meta.len() == 0 {
        bail!("file is empty: {}", path.display());
    }
    Ok(())
}

pub(crate) fn ensure_contains_line_start(path: &Path, prefix: &str) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read: {}", path.display()))?;
    if content.lines().any(|line| line.starts_with(prefix)) {
        return Ok(());
    }
    bail!(
        "expected line starting with '{}' not found in {}",
        prefix,
        path.display()
    )
}

pub(crate) fn ensure_contains_substr(path: &Path, pattern: &str) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read: {}", path.display()))?;
    if content.contains(pattern) {
        return Ok(());
    }
    bail!("expected '{}' not found in {}", pattern, path.display())
}

pub(crate) struct PythonExec {
    pub(crate) program: &'static str,
    pub(crate) prefix_args: &'static [&'static str],
}

pub(crate) fn run_python(python: &PythonExec, script_args: &[&str], quiet: bool) -> Result<()> {
    let mut all_args: Vec<&str> = python.prefix_args.to_vec();
    all_args.extend_from_slice(script_args);
    run_cmd(
        python.program,
        &all_args,
        quiet,
        &[("PYTHONUTF8", "1"), ("PYTHONIOENCODING", "utf-8")],
        &[0],
    )
}

pub(crate) fn detect_python() -> Result<PythonExec> {
    let candidates = [
        PythonExec {
            program: "python3",
            prefix_args: &[],
        },
        PythonExec {
            program: "python",
            prefix_args: &[],
        },
        PythonExec {
            program: "py",
            prefix_args: &["-3"],
        },
    ];

    for candidate in candidates {
        let mut cmd = Command::new(candidate.program);
        cmd.args(candidate.prefix_args);
        let status = cmd
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if let Ok(s) = status {
            if s.success() {
                return Ok(candidate);
            }
        }
    }
    bail!("python executable not found (tried: python3, python, py -3)")
}
