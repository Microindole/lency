use super::case_scan::parse_case_meta;
use crate::helpers::{
    ensure_contains_line_start, ensure_contains_substr, ensure_file_non_empty, resolve_exec,
    run_cmd,
};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

pub(crate) fn run_selfhost_basic_tests(self_host_out: &Path) -> Result<()> {
    crate::helpers::step("3. Running Compiled Self-host Lencyc Basic Tests", || {
        run_cmd(self_host_out, &[], false, &[], &[0])
    })
}

pub(crate) fn compile_selfhost_main_entry(
    rust_lency_exec: &Path,
    self_host_main_entry: &Path,
    self_host_out_dir: &Path,
    self_host_main_out_name: &str,
) -> Result<PathBuf> {
    crate::helpers::step("4. Compiling Self-host Main Pipeline Entry", || {
        if !self_host_main_entry.exists() {
            bail!(
                "cannot find self-host main entry file: {}",
                self_host_main_entry.display()
            );
        }
        run_cmd(
            rust_lency_exec,
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
    resolve_exec(&self_host_out_dir.join(self_host_main_out_name))
}

pub(crate) fn run_selfhost_main_pipeline(
    self_host_main_out: &Path,
    emit_path: &Path,
) -> Result<()> {
    crate::helpers::step("5. Running Self-host Main Pipeline", || {
        run_cmd(
            self_host_main_out,
            &[
                "lencyc/driver/pipeline_sample.lcy",
                "-o",
                &emit_path.to_string_lossy(),
            ],
            false,
            &[],
            &[0],
        )
    })
}

pub(crate) fn verify_selfhost_main_emit_output(emit_path: &Path) -> Result<()> {
    crate::helpers::step("6. Verifying Self-host Main Emit Output", || {
        ensure_file_non_empty(emit_path)?;
        ensure_contains_line_start(emit_path, "AST[0]:")?;
        Ok(())
    })
}

pub(crate) fn run_lir_emit_regression_cases(
    self_host_main_out: &Path,
    self_host_out_dir: &Path,
    lir_cases: &[PathBuf],
) -> Result<()> {
    crate::helpers::step("7. Running Self-host LIR Emit Regression Cases", || {
        for case_file in lir_cases {
            let case_name = case_file
                .file_stem()
                .and_then(|s| s.to_str())
                .context("invalid LIR case filename")?;
            let case_out = self_host_out_dir.join(format!("{case_name}.lir"));

            run_cmd(
                self_host_main_out,
                &[
                    &case_file.to_string_lossy(),
                    "--emit-lir",
                    "-o",
                    &case_out.to_string_lossy(),
                ],
                true,
                &[],
                &[0],
            )?;

            ensure_file_non_empty(&case_out)?;
            ensure_contains_line_start(&case_out, "; lencyc-lir v0")?;
            ensure_contains_substr(&case_out, "func ")?;
        }
        Ok(())
    })
}

pub(crate) fn run_rust_lir_to_llvm_smoke(
    self_host_main_out: &Path,
    rust_lency_exec: &Path,
    self_host_out_dir: &Path,
    lir_e2e_case: &Path,
) -> Result<()> {
    let lir_e2e_out = self_host_out_dir.join("lir_e2e_case.lir");
    let lir_e2e_bin_name = "lir_e2e_case";
    crate::helpers::step("8. Running Rust LIR->LLVM Build Smoke Test", || {
        run_cmd(
            self_host_main_out,
            &[
                &lir_e2e_case.to_string_lossy(),
                "--emit-lir",
                "-o",
                &lir_e2e_out.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            rust_lency_exec,
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
    })
}

pub(crate) fn run_one_step_build_flow(
    self_host_main_out: &Path,
    rust_lency_exec: &Path,
    self_host_out_dir: &Path,
    self_host_main_entry: &Path,
    self_host_main_out_name: &str,
    lir_e2e_case: &Path,
) -> Result<()> {
    crate::helpers::step("9. Running Self-host One-step Build Flow", || {
        let flow_bin_name = "selfhost_flow_case";
        let flow_emit = self_host_out_dir.join("selfhost_flow_case.lir");
        let flow_bin = self_host_out_dir.join(flow_bin_name);

        run_cmd(
            rust_lency_exec,
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
            self_host_main_out,
            &[
                &lir_e2e_case.to_string_lossy(),
                "--emit-lir",
                "-o",
                &flow_emit.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            rust_lency_exec,
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
    })
}

pub(crate) fn run_runtime_cases(
    self_host_main_out: &Path,
    rust_lency_exec: &Path,
    self_host_out_dir: &Path,
    runtime_cases: &[PathBuf],
) -> Result<()> {
    crate::helpers::step("10. Running Self-host Runtime Cases", || {
        for case in runtime_cases {
            let meta = parse_case_meta(case)?;
            let case_name = case
                .file_stem()
                .and_then(|s| s.to_str())
                .context("invalid runtime case filename")?;
            let output_name = format!("{case_name}.run.out");
            let output_base = self_host_out_dir.join(&output_name);
            let emit_path = self_host_out_dir.join(format!("{case_name}.selfhost.lir"));

            run_cmd(
                self_host_main_out,
                &[
                    &case.to_string_lossy(),
                    "--emit-lir",
                    "-o",
                    &emit_path.to_string_lossy(),
                ],
                true,
                &[],
                &[0],
            )?;
            run_cmd(
                rust_lency_exec,
                &[
                    "build",
                    &emit_path.to_string_lossy(),
                    "-o",
                    &output_name,
                    "--out-dir",
                    &self_host_out_dir.to_string_lossy(),
                ],
                true,
                &[],
                &[0],
            )?;
            let out_exe = resolve_exec(&output_base)?;
            let run_args_ref: Vec<&str> = meta.run_args.iter().map(String::as_str).collect();
            println!("runtime case: {}", case.display());
            println!("runtime lir: {}", emit_path.display());
            println!("runtime exe: {}", out_exe.display());
            run_cmd(&out_exe, &run_args_ref, false, &[], &[meta.expect_exit])?;
        }
        Ok(())
    })
}
