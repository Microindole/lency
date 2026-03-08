use crate::helpers::{ensure_contains_line_start, ensure_file_non_empty, run_cmd_exit_code};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub(crate) struct ExampleCaseMeta {
    pub(crate) expect_error: bool,
    pub(crate) expect_exit: i32,
    pub(crate) run_args: Vec<String>,
    pub(crate) selfhost_e2e: bool,
}

pub(crate) fn collect_lcy_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = fs::read_dir(dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("failed to list directory entries: {}", dir.display()))?;
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("lcy") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    walk(root, &mut files)?;
    Ok(files)
}

pub(crate) fn parse_case_meta(path: &Path) -> Result<ExampleCaseMeta> {
    let mut meta = ExampleCaseMeta::default();
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    for line in content.lines().take(8) {
        let text = line.trim();
        if text.contains("@expect-error") {
            meta.expect_error = true;
        }
        if let Some(value) = text.strip_prefix("// @expect-exit:") {
            let v = value.trim();
            meta.expect_exit = v.parse::<i32>().with_context(|| {
                format!("invalid @expect-exit value '{v}' in {}", path.display())
            })?;
        }
        if let Some(value) = text.strip_prefix("// @run-args:") {
            meta.run_args = value.split_whitespace().map(|s| s.to_string()).collect();
        }
        if text.contains("@selfhost-e2e") {
            meta.selfhost_e2e = true;
        }
    }
    Ok(meta)
}

fn rel_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn slugify_case_name(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub(crate) fn run_recursive_selfhost_cases(
    self_host_main_out: &Path,
    self_host_out_dir: &Path,
) -> Result<()> {
    let mut files = Vec::new();
    for root in [
        Path::new("tests/example/lir"),
        Path::new("tests/example/parser"),
        Path::new("tests/example/runtime"),
    ] {
        files.extend(collect_lcy_files(root)?);
    }
    if files.is_empty() {
        bail!("no .lcy case files found under tests/example/lir, tests/example/parser, tests/example/runtime");
    }

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut expected_error = 0usize;

    for case in files {
        let meta = parse_case_meta(&case)?;
        let out_name = format!("{}.scan.lir", slugify_case_name(&case));
        let out_path = self_host_out_dir.join(out_name);
        let code = run_cmd_exit_code(
            self_host_main_out,
            &[
                &case.to_string_lossy(),
                "--emit-lir",
                "-o",
                &out_path.to_string_lossy(),
            ],
            true,
        )?;
        let rel = rel_display(&case);
        if code == 0 {
            if meta.expect_error {
                println!("WARN  {rel} (expected failure but passed)");
                fail += 1;
            } else {
                ensure_file_non_empty(&out_path)?;
                ensure_contains_line_start(&out_path, "; lencyc-lir v0")?;
                println!("PASS  {rel}");
                pass += 1;
            }
        } else if meta.expect_error {
            println!("XFAIL {rel} (expected error)");
            expected_error += 1;
        } else {
            println!("FAIL  {rel}");
            fail += 1;
        }
    }

    println!(
        "Recursive selfhost case summary: pass={}, expected_error={}, fail={}",
        pass, expected_error, fail
    );
    if fail > 0 {
        bail!(
            "recursive selfhost cases failed: {} unexpected failures",
            fail
        );
    }
    Ok(())
}

pub(crate) fn select_lir_e2e_case(lir_cases: &[PathBuf]) -> Result<PathBuf> {
    for case in lir_cases {
        let meta = parse_case_meta(case)?;
        if meta.selfhost_e2e {
            return Ok(case.clone());
        }
    }
    lir_cases
        .first()
        .cloned()
        .context("no LIR case available for selfhost e2e")
}
