use super::{check_docs_quick, check_lency, check_rust};
use crate::helpers::step;
use anyhow::{bail, Context, Result};
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckScope {
    RustOnly,
    LencyOnly,
    Both,
    DocsOnly,
    None,
}

fn is_rust_change(path: &str) -> bool {
    path.starts_with("crates/")
        || path.starts_with("tests/integration/")
        || path == "Cargo.toml"
        || path == "Cargo.lock"
        || path == "scripts/linux/run_checks.sh"
        || path == "scripts/win/run_checks.ps1"
        || path == "scripts/linux/run_lcy_tests.sh"
        || path == "scripts/win/run_lcy_tests.ps1"
}

fn is_lency_change(path: &str) -> bool {
    path.starts_with("lencyc/")
        || path.starts_with("tests/example/")
        || path.starts_with("lib/")
        || path.starts_with("xtask/")
        || path == "scripts/linux/run_lency_checks.sh"
        || path == "scripts/win/run_lency_checks.ps1"
        || path == "scripts/linux/lency_selfhost_build.sh"
        || path == "scripts/linux/lency_selfhost_run.sh"
        || path == "scripts/win/lency_selfhost_build.ps1"
        || path == "scripts/win/lency_selfhost_run.ps1"
}

fn is_docs_change(path: &str) -> bool {
    path.starts_with("docs/")
        || path.ends_with(".md")
        || path.ends_with(".txt")
}

fn detect_check_scope_from_status(stdout: &str) -> CheckScope {
    let mut has_rust = false;
    let mut has_lency = false;
    let mut has_docs = false;

    for line in stdout.lines() {
        if line.len() < 4 {
            continue;
        }
        let raw = &line[3..];
        let path = if let Some((_, new_path)) = raw.split_once(" -> ") {
            new_path
        } else {
            raw
        };
        let normalized = path.replace('\\', "/");

        if is_rust_change(&normalized) {
            has_rust = true;
        }
        if is_lency_change(&normalized) {
            has_lency = true;
        }
        if is_docs_change(&normalized) {
            has_docs = true;
        }
    }

    match (has_rust, has_lency) {
        (true, true) => CheckScope::Both,
        (true, false) => CheckScope::RustOnly,
        (false, true) => CheckScope::LencyOnly,
        (false, false) => {
            if has_docs {
                CheckScope::DocsOnly
            } else {
                CheckScope::None
            }
        }
    }
}

fn detect_check_scope_from_git_status() -> Result<CheckScope> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("failed to run: git status --porcelain")?;
    if !output.status.success() {
        bail!(
            "git status --porcelain failed with exit code {:?}",
            output.status.code()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(detect_check_scope_from_status(&stdout))
}

pub(crate) fn auto_check() -> Result<()> {
    let mut scope = CheckScope::None;
    step("Detecting changed scope from git status", || {
        scope = detect_check_scope_from_git_status()?;
        println!("Detected scope: {scope:?}");
        Ok(())
    })?;

    match scope {
        CheckScope::RustOnly => check_rust(),
        CheckScope::LencyOnly => check_lency(),
        CheckScope::Both => {
            check_rust()?;
            check_lency()
        }
        CheckScope::DocsOnly => {
            println!("Docs-only changes detected, running quick check mode.");
            check_docs_quick()
        }
        CheckScope::None => {
            println!("No Rust/Lency scoped changes detected, fallback to check-lency.");
            check_lency()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_check_scope_from_status, CheckScope};

    #[test]
    fn scope_docs_only() {
        let status = " M docs/development/status.md\n?? docs/notes.md\n";
        assert_eq!(detect_check_scope_from_status(status), CheckScope::DocsOnly);
    }

    #[test]
    fn scope_none_for_non_docs_misc_file() {
        let status = " M README\n";
        assert_eq!(detect_check_scope_from_status(status), CheckScope::None);
    }

    #[test]
    fn scope_both_when_rust_and_lency() {
        let status = " M crates/lency_cli/src/main.rs\n M lencyc/sema/resolver.lcy\n";
        assert_eq!(detect_check_scope_from_status(status), CheckScope::Both);
    }
}
