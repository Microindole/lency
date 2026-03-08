mod case_scan;
mod selfhost_flow;

use crate::helpers::{
    detect_python, resolve_exec, run_cmd, run_cmd_capture, run_cmd_exit_code, run_python, step,
};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use case_scan::{collect_lcy_files, run_recursive_selfhost_cases, select_lir_e2e_case};
use selfhost_flow::{
    compile_selfhost_main_entry, run_lir_emit_regression_cases, run_one_step_build_flow,
    run_runtime_cases, run_rust_lir_to_llvm_smoke, run_selfhost_basic_tests,
    run_selfhost_main_pipeline, verify_selfhost_main_emit_output,
};

pub(crate) fn check_lency() -> Result<()> {
    let python = detect_python()?;
    let rust_lency_exec_base = Path::new("target").join("release").join("lencyc");
    let self_host_entry = Path::new("tests/example/selfhost/driver/test_entry.lcy");
    let self_host_out_dir = Path::new("target/lencyc_selfhost");
    let self_host_out_name = "lencyc_test";
    let self_host_main_entry = Path::new("lencyc/driver/main.lcy");
    let self_host_main_out_name = "lencyc_main";
    let self_host_main_emit = Path::new("lencyc_selfhost_ast.txt");

    step(
        "1. Compiling Rust Host Compiler (lency_cli + lency_runtime)",
        || {
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
            )
        },
    )?;

    let rust_lency_exec = resolve_exec(&rust_lency_exec_base)?;

    step(
        "1.5. Running Meta Checks (TODOs, File Size, Naming)",
        || {
            run_python(
                &python,
                &["scripts/check_todos.py", "--scope", "lency"],
                false,
            )?;
            run_python(
                &python,
                &["scripts/check_file_size.py", "--scope", "lency"],
                false,
            )?;
            run_python(&python, &["scripts/check_lencyc_meta.py"], false)
        },
    )?;

    step("1.6. Running Entry Syntax Checks for lencyc/", || {
        let help_text = run_cmd_capture(&rust_lency_exec, &["build", "--help"])?;
        if help_text.contains("--check-only") {
            for entry in [
                "tests/example/selfhost/driver/test_entry.lcy",
                "lencyc/driver/main.lcy",
            ] {
                if !Path::new(entry).exists() {
                    bail!("missing check entry: {entry}");
                }
                let code =
                    run_cmd_exit_code(&rust_lency_exec, &["build", entry, "--check-only"], true)?;
                if code != 0 {
                    println!(
                        "Warning: check-only failed for {entry} (exit {code}), fallback to full build validation."
                    );
                }
            }
        } else {
            println!("Skipped entry syntax checks: '--check-only' is not supported.");
        }
        Ok(())
    })?;

    let mut selfhost_ready = true;
    step(
        "2. Compiling Lency-written Compiler (Self-host Lencyc)",
        || {
            if !self_host_entry.exists() {
                bail!(
                    "cannot find self-host entry file: {}",
                    self_host_entry.display()
                );
            }
            fs::create_dir_all(self_host_out_dir)
                .with_context(|| format!("failed to create {}", self_host_out_dir.display()))?;
            let code = run_cmd_exit_code(
                &rust_lency_exec,
                &[
                    "build",
                    &self_host_entry.to_string_lossy(),
                    "-o",
                    self_host_out_name,
                    "--out-dir",
                    &self_host_out_dir.to_string_lossy(),
                ],
                false,
            )?;
            if code != 0 {
                if cfg!(windows) {
                    println!(
                        "Warning: self-host build failed on Windows (exit {code}), skip remaining self-host steps."
                    );
                    selfhost_ready = false;
                    return Ok(());
                }
                bail!("self-host build failed with exit code {code}");
            }
            Ok(())
        },
    )?;

    if !selfhost_ready {
        println!("\nLency checks finished with Windows fallback: steps 3-11 skipped.");
        return Ok(());
    }

    let self_host_out = resolve_exec(&self_host_out_dir.join(self_host_out_name))?;
    run_selfhost_basic_tests(&self_host_out)?;

    let self_host_main_out = compile_selfhost_main_entry(
        &rust_lency_exec,
        self_host_main_entry,
        self_host_out_dir,
        self_host_main_out_name,
    )?;
    run_selfhost_main_pipeline(&self_host_main_out, self_host_main_emit)?;
    verify_selfhost_main_emit_output(self_host_main_emit)?;
    step(
        "6.5. Running recursive tests/example selfhost cases",
        || run_recursive_selfhost_cases(&self_host_main_out, self_host_out_dir),
    )?;

    let lir_cases = collect_lcy_files(Path::new("tests/example/lir"))?;
    if lir_cases.is_empty() {
        bail!("no LIR regression cases found under tests/example/lir");
    }
    run_lir_emit_regression_cases(&self_host_main_out, self_host_out_dir, &lir_cases)?;

    let lir_e2e_case = select_lir_e2e_case(&lir_cases)?;
    run_rust_lir_to_llvm_smoke(
        &self_host_main_out,
        &rust_lency_exec,
        self_host_out_dir,
        &lir_e2e_case,
    )?;
    run_one_step_build_flow(
        &self_host_main_out,
        &rust_lency_exec,
        self_host_out_dir,
        self_host_main_entry,
        self_host_main_out_name,
        &lir_e2e_case,
    )?;

    let runtime_cases = collect_lcy_files(Path::new("tests/example/runtime"))?;
    if runtime_cases.is_empty() {
        bail!("no runtime cases found under tests/example/runtime");
    }
    run_runtime_cases(
        &self_host_main_out,
        &rust_lency_exec,
        self_host_out_dir,
        &runtime_cases,
    )?;

    println!("\nAll Lency-side checks passed.");
    Ok(())
}
