use super::{prepare_rust_lency_cli, SelfhostBuildOptions};
use crate::helpers::{resolve_exec, run_cmd};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn selfhost_build_impl(opts: &SelfhostBuildOptions) -> Result<PathBuf> {
    let rust_lency_exec = prepare_rust_lency_cli()?;
    let self_host_out_dir = Path::new("target/lencyc_selfhost");
    let self_host_main_entry = Path::new("lencyc/driver/main.lcy");
    let self_host_main_out_name = "lencyc_main";

    if !opts.input_file.exists() {
        bail!("input file not found: {}", opts.input_file.display());
    }

    fs::create_dir_all(self_host_out_dir)
        .with_context(|| format!("failed to create {}", self_host_out_dir.display()))?;
    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("failed to create {}", opts.out_dir.display()))?;

    println!("[1/4] building rust host compiler ...");
    println!("[2/4] building self-host compiler entry ...");
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

    let self_host_main = resolve_exec(&self_host_out_dir.join(self_host_main_out_name))?;
    let emit_name = format!(
        "{}.selfhost.lir",
        opts.input_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
    );
    let emit_path = self_host_out_dir.join(emit_name);

    println!("[3/4] emitting LIR from self-host compiler ...");
    run_cmd(
        &self_host_main,
        &[
            &opts.input_file.to_string_lossy(),
            "--emit-lir",
            "-o",
            &emit_path.to_string_lossy(),
        ],
        true,
        &[],
        &[0],
    )?;

    println!("[4/4] building executable from emitted LIR ...");
    let mut build_args = vec![
        "build".to_string(),
        emit_path.to_string_lossy().to_string(),
        "-o".to_string(),
        opts.output_name.clone(),
        "--out-dir".to_string(),
        opts.out_dir.to_string_lossy().to_string(),
    ];
    if opts.check_only {
        build_args.push("--check-only".to_string());
    }
    if opts.release {
        build_args.push("--release".to_string());
    }

    let build_args_ref: Vec<&str> = build_args.iter().map(String::as_str).collect();
    run_cmd(&rust_lency_exec, &build_args_ref, true, &[], &[0])?;

    Ok(opts.out_dir.join(&opts.output_name))
}
