use super::{SelfhostBuildOptions, SelfhostRunOptions};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

pub(crate) fn parse_selfhost_build_args(args: &[String]) -> Result<SelfhostBuildOptions> {
    let mut input_file: Option<PathBuf> = None;
    let mut output_name: Option<String> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut check_only = false;
    let mut release = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                let Some(v) = args.get(i + 1) else {
                    bail!("{} requires a value", args[i]);
                };
                output_name = Some(v.clone());
                i += 2;
            }
            "--out-dir" => {
                let Some(v) = args.get(i + 1) else {
                    bail!("--out-dir requires a value");
                };
                out_dir = Some(PathBuf::from(v));
                i += 2;
            }
            "--check-only" => {
                check_only = true;
                i += 1;
            }
            "--release" => {
                release = true;
                i += 1;
            }
            s if s.starts_with('-') => bail!("unknown option: {s}"),
            other => {
                if input_file.is_some() {
                    bail!("multiple input files are not supported");
                }
                input_file = Some(PathBuf::from(other));
                i += 1;
            }
        }
    }

    let input_file = input_file.context("missing input file")?;
    let output_name = output_name.unwrap_or_else(|| {
        input_file
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| format!("{s}.out"))
            .unwrap_or_else(|| "a.out".to_string())
    });
    let out_dir = out_dir.unwrap_or_else(|| PathBuf::from("target/lencyc_selfhost"));

    Ok(SelfhostBuildOptions {
        input_file,
        output_name,
        out_dir,
        check_only,
        release,
    })
}

pub(crate) fn parse_selfhost_run_args(args: &[String]) -> Result<SelfhostRunOptions> {
    let mut input_file: Option<PathBuf> = None;
    let mut out_dir = PathBuf::from("target/lencyc_selfhost");
    let mut release = false;
    let mut expect_exit: Option<i32> = None;
    let mut program_args: Vec<String> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--release" => {
                release = true;
                i += 1;
            }
            "--out-dir" => {
                let Some(v) = args.get(i + 1) else {
                    bail!("--out-dir requires a value");
                };
                out_dir = PathBuf::from(v);
                i += 2;
            }
            "--expect-exit" => {
                let Some(v) = args.get(i + 1) else {
                    bail!("--expect-exit requires a value");
                };
                expect_exit = Some(
                    v.parse::<i32>()
                        .with_context(|| format!("invalid --expect-exit value: {v}"))?,
                );
                i += 2;
            }
            "--" => {
                program_args.extend(args[(i + 1)..].iter().cloned());
                break;
            }
            s if s.starts_with('-') => bail!("unknown option: {s}"),
            other => {
                if input_file.is_none() {
                    input_file = Some(PathBuf::from(other));
                } else {
                    program_args.push(other.to_string());
                }
                i += 1;
            }
        }
    }

    let input_file = input_file.context("missing input file")?;
    Ok(SelfhostRunOptions {
        input_file,
        out_dir,
        release,
        expect_exit,
        program_args,
    })
}
