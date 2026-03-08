mod args;
mod build;
mod run;
mod usage;

use crate::helpers::{resolve_exec, run_cmd};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(crate) use build::selfhost_build_impl;
pub(crate) use run::selfhost_run_impl;

#[derive(Debug)]
pub(crate) struct SelfhostBuildOptions {
    pub(crate) input_file: PathBuf,
    pub(crate) output_name: String,
    pub(crate) out_dir: PathBuf,
    pub(crate) check_only: bool,
    pub(crate) release: bool,
}

#[derive(Debug)]
pub(crate) struct SelfhostRunOptions {
    pub(crate) input_file: PathBuf,
    pub(crate) out_dir: PathBuf,
    pub(crate) release: bool,
    pub(crate) expect_exit: Option<i32>,
    pub(crate) program_args: Vec<String>,
}

pub(crate) fn selfhost_build_from_args(args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage::print_selfhost_build_usage();
        return Ok(());
    }

    let opts = args::parse_selfhost_build_args(args)?;
    let result = selfhost_build_impl(&opts)?;
    if opts.check_only {
        println!("self-host check-only passed: {}", opts.input_file.display());
    } else {
        println!("self-host build succeeded: {}", result.display());
    }
    Ok(())
}

pub(crate) fn selfhost_run_from_args(args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage::print_selfhost_run_usage();
        return Ok(());
    }
    let opts = args::parse_selfhost_run_args(args)?;
    selfhost_run_impl(&opts)
}

pub(crate) fn prepare_rust_lency_cli() -> Result<PathBuf> {
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
        true,
        &[],
        &[0],
    )?;
    let base = Path::new("target").join("release").join("lencyc");
    resolve_exec(&base)
}
