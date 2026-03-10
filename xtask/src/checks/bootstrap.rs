use crate::helpers::{resolve_exec, run_cmd, step};
use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::Path;

pub(crate) fn bootstrap_check() -> Result<()> {
    let rust_lency_exec_base = Path::new("target").join("release").join("lencyc");
    let compiler_source = Path::new("lencyc/driver/main.lcy");
    let sample_input = Path::new("tests/example/lir/lencyc_lir_exit0.lcy");
    let out_dir = Path::new("target/bootstrap_check");

    let stage1_name = "lencyc_stage1";
    let stage2_name = "lencyc_stage2";
    let stage3_name = "lencyc_stage3";

    let stage2_compiler_lir = out_dir.join("stage2_compiler.lir");
    let stage3_compiler_lir = out_dir.join("stage3_compiler.lir");
    let stage2_sample_lir = out_dir.join("stage2_sample.lir");
    let stage3_sample_lir = out_dir.join("stage3_sample.lir");

    let strict_binary = env::var("LENCY_BOOTSTRAP_STRICT_BINARY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    step("B1. Compiling Rust Host Compiler (release lencyc)", || {
        run_cmd(
            "cargo",
            &["build", "--release", "-p", "lency_cli"],
            false,
            &[],
            &[0],
        )
    })?;
    let rust_lency_exec = resolve_exec(&rust_lency_exec_base)?;

    fs::create_dir_all(out_dir).with_context(|| {
        format!(
            "failed to create bootstrap output dir: {}",
            out_dir.display()
        )
    })?;

    step("B2. Building Stage1 Compiler with Rust Host", || {
        run_cmd(
            &rust_lency_exec,
            &[
                "build",
                &compiler_source.to_string_lossy(),
                "-o",
                stage1_name,
                "--out-dir",
                &out_dir.to_string_lossy(),
            ],
            false,
            &[],
            &[0],
        )
    })?;
    let stage1_exec = resolve_exec(&out_dir.join(stage1_name))?;

    step(
        "B3. Building Stage2 Compiler from Stage1 emitted LIR",
        || {
            build_next_stage(
                &stage1_exec,
                &rust_lency_exec,
                compiler_source,
                &stage2_compiler_lir,
                out_dir,
                stage2_name,
            )
        },
    )?;
    let stage2_exec = resolve_exec(&out_dir.join(stage2_name))?;

    step(
        "B4. Building Stage3 Compiler from Stage2 emitted LIR",
        || {
            build_next_stage(
                &stage2_exec,
                &rust_lency_exec,
                compiler_source,
                &stage3_compiler_lir,
                out_dir,
                stage3_name,
            )
        },
    )?;
    let stage3_exec = resolve_exec(&out_dir.join(stage3_name))?;

    step("B5. Comparing Stage2/Stage3 compiler LIR", || {
        compare_files_exact(
            &stage2_compiler_lir,
            &stage3_compiler_lir,
            "stage2_compiler.lir",
            "stage3_compiler.lir",
        )
    })?;

    step("B6. Comparing Stage2/Stage3 sample output LIR", || {
        emit_lir_with_compiler(&stage2_exec, sample_input, &stage2_sample_lir)?;
        emit_lir_with_compiler(&stage3_exec, sample_input, &stage3_sample_lir)?;
        compare_files_exact(
            &stage2_sample_lir,
            &stage3_sample_lir,
            "stage2_sample.lir",
            "stage3_sample.lir",
        )
    })?;

    step("B7. Running Stage2/Stage3 smoke pipeline", || {
        run_cmd(
            &stage2_exec,
            &[
                &sample_input.to_string_lossy(),
                "--emit-lir",
                "-o",
                &stage2_sample_lir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )?;
        run_cmd(
            &stage3_exec,
            &[
                &sample_input.to_string_lossy(),
                "--emit-lir",
                "-o",
                &stage3_sample_lir.to_string_lossy(),
            ],
            true,
            &[],
            &[0],
        )
    })?;

    step("B8. Optional strict binary comparison", || {
        if !strict_binary {
            println!("skip strict binary compare (set LENCY_BOOTSTRAP_STRICT_BINARY=1 to enable)");
            // TODO: keep strict mode opt-in until toolchain reproducibility is fully controlled in CI.
            return Ok(());
        }
        compare_files_exact(
            &stage2_exec,
            &stage3_exec,
            "stage2 compiler binary",
            "stage3 compiler binary",
        )
    })?;

    println!("\nBootstrap check passed: stage chain converged at LIR level.");
    Ok(())
}

fn build_next_stage(
    compiler_exec: &Path,
    rust_lency_exec: &Path,
    compiler_source: &Path,
    emit_lir_path: &Path,
    out_dir: &Path,
    output_name: &str,
) -> Result<()> {
    emit_lir_with_compiler(compiler_exec, compiler_source, emit_lir_path)?;
    run_cmd(
        rust_lency_exec,
        &[
            "build",
            &emit_lir_path.to_string_lossy(),
            "-o",
            output_name,
            "--out-dir",
            &out_dir.to_string_lossy(),
        ],
        false,
        &[],
        &[0],
    )
}

fn emit_lir_with_compiler(
    compiler_exec: &Path,
    input_file: &Path,
    output_lir: &Path,
) -> Result<()> {
    run_cmd(
        compiler_exec,
        &[
            &input_file.to_string_lossy(),
            "--emit-lir",
            "-o",
            &output_lir.to_string_lossy(),
        ],
        false,
        &[],
        &[0],
    )
}

fn compare_files_exact(left: &Path, right: &Path, left_name: &str, right_name: &str) -> Result<()> {
    let left_bytes = fs::read(left)
        .with_context(|| format!("failed to read comparison file: {}", left.display()))?;
    let right_bytes = fs::read(right)
        .with_context(|| format!("failed to read comparison file: {}", right.display()))?;
    if left_bytes != right_bytes {
        bail!(
            "bootstrap mismatch: {} != {} ({} vs {})",
            left_name,
            right_name,
            left.display(),
            right.display()
        );
    }
    Ok(())
}
