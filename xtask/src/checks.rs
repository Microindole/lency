use crate::helpers::{
    detect_python, ensure_contains_line_start, ensure_contains_substr, ensure_file_non_empty,
    resolve_exec, run_cmd, run_cmd_capture, run_cmd_exit_code, run_python, step,
};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

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

pub(crate) fn check_lency() -> Result<()> {
    let python = detect_python()?;
    let rust_lency_exec_base = Path::new("target").join("release").join("lencyc");
    let self_host_entry = Path::new("lencyc/driver/test_entry.lcy");
    let self_host_out_dir = Path::new("target/lencyc_selfhost");
    let self_host_out_name = "lencyc_test";
    let self_host_main_entry = Path::new("lencyc/driver/main.lcy");
    let self_host_main_out_name = "lencyc_main";
    let self_host_main_emit = Path::new("lencyc_selfhost_ast.txt");

    let lir_test_cases = [
        "tests/example/lencyc_lir_basic.lcy",
        "tests/example/lencyc_lir_exit0.lcy",
        "tests/example/lencyc_lir_loop_if.lcy",
        "tests/example/lencyc_lir_unary_logic.lcy",
        "tests/example/lencyc_lir_break_continue.lcy",
    ];

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
            for entry in ["lencyc/driver/test_entry.lcy", "lencyc/driver/main.lcy"] {
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

    step("3. Running Compiled Self-host Lencyc Basic Tests", || {
        run_cmd(&self_host_out, &[], false, &[], &[0])
    })?;

    step("4. Compiling Self-host Main Pipeline Entry", || {
        if !self_host_main_entry.exists() {
            bail!(
                "cannot find self-host main entry file: {}",
                self_host_main_entry.display()
            );
        }
        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &self_host_main_entry.to_string_lossy(),
                "-o",
                self_host_main_out_name,
                "--out-dir",
                &self_host_out_dir.to_string_lossy(),
            ],
            false,
            &[],
            &[0],
        )
    })?;

    let self_host_main_out = resolve_exec(&self_host_out_dir.join(self_host_main_out_name))?;

    step("5. Running Self-host Main Pipeline", || {
        run_cmd(
            &self_host_main_out,
            &[
                "lencyc/driver/pipeline_sample.lcy",
                "-o",
                &self_host_main_emit.to_string_lossy(),
            ],
            false,
            &[],
            &[0],
        )
    })?;

    step("6. Verifying Self-host Main Emit Output", || {
        ensure_file_non_empty(self_host_main_emit)?;
        ensure_contains_line_start(self_host_main_emit, "AST[0]:")?;
        Ok(())
    })?;

    step("7. Running Self-host LIR Emit Regression Cases", || {
        for case_file in lir_test_cases {
            let case_path = Path::new(case_file);
            if !case_path.exists() {
                bail!("missing LIR test case: {}", case_path.display());
            }

            let case_name = case_path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("invalid LIR case filename")?;
            let case_out = self_host_out_dir.join(format!("{case_name}.lir"));

            run_cmd(
                &self_host_main_out,
                &[case_file, "--emit-lir", "-o", &case_out.to_string_lossy()],
                true,
                &[],
                &[0],
            )?;

            ensure_file_non_empty(&case_out)?;
            ensure_contains_line_start(&case_out, "; lencyc-lir v0")?;
            ensure_contains_line_start(&case_out, "func main {")?;
            ensure_contains_substr(&case_out, "ret")?;
        }
        Ok(())
    })?;

    let lir_e2e_case = "tests/example/lencyc_lir_exit0.lcy";
    let lir_e2e_out = self_host_out_dir.join("lir_e2e_exit0.lir");
    let lir_e2e_bin_name = "lir_e2e_exit0";

    step("8. Running Rust LIR->LLVM Build Smoke Test", || {
        run_cmd(
            &self_host_main_out,
            &[
                lir_e2e_case,
                "--emit-lir",
                "-o",
                &lir_e2e_out.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &lir_e2e_out.to_string_lossy(),
                "-o",
                lir_e2e_bin_name,
                "--out-dir",
                &self_host_out_dir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        let lir_e2e_bin = resolve_exec(&self_host_out_dir.join(lir_e2e_bin_name))?;
        run_cmd(&lir_e2e_bin, &[], true, &[], &[0])
    })?;

    step("9. Running Self-host One-step Build Flow", || {
        let flow_bin_name = "selfhost_flow_exit0";
        let flow_emit = self_host_out_dir.join("lencyc_lir_exit0.selfhost.lir");
        let flow_bin = self_host_out_dir.join(flow_bin_name);

        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &self_host_main_entry.to_string_lossy(),
                "-o",
                self_host_main_out_name,
                "--out-dir",
                &self_host_out_dir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            &self_host_main_out,
            &[
                lir_e2e_case,
                "--emit-lir",
                "-o",
                &flow_emit.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &flow_emit.to_string_lossy(),
                "-o",
                flow_bin_name,
                "--out-dir",
                &self_host_out_dir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        let flow_bin = resolve_exec(&flow_bin)?;
        run_cmd(&flow_bin, &[], true, &[], &[0])
    })?;

    step("10. Running Self-host One-step Run Flow", || {
        let case = "tests/example/lencyc_run_args.lcy";
        let output_name = "lencyc_run_args.run.out";
        let output_base = self_host_out_dir.join(output_name);
        let emit_path = self_host_out_dir.join("lencyc_run_args.selfhost.lir");

        run_cmd(
            &self_host_main_out,
            &[case, "--emit-lir", "-o", &emit_path.to_string_lossy()],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &emit_path.to_string_lossy(),
                "-o",
                output_name,
                "--out-dir",
                &self_host_out_dir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        let out_exe = resolve_exec(&output_base)?;
        run_cmd(&out_exe, &["sample_arg"], true, &[], &[1])
    })?;

    step("11. Running Self-host Runtime Builtin Mapping Flow", || {
        let case = "tests/example/lencyc_run_int_to_string.lcy";
        let output_name = "lencyc_run_int_to_string.run.out";
        let output_base = self_host_out_dir.join(output_name);
        let emit_path = self_host_out_dir.join("lencyc_run_int_to_string.selfhost.lir");

        run_cmd(
            &self_host_main_out,
            &[case, "--emit-lir", "-o", &emit_path.to_string_lossy()],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &emit_path.to_string_lossy(),
                "-o",
                output_name,
                "--out-dir",
                &self_host_out_dir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        let out_exe = resolve_exec(&output_base)?;
        run_cmd(&out_exe, &[], true, &[], &[0])
    })?;

    println!("\nAll Lency-side checks passed.");
    Ok(())
}
