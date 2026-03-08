use super::{selfhost_build_impl, SelfhostBuildOptions, SelfhostRunOptions};
use crate::helpers::{resolve_exec, run_cmd_exit_code};
use anyhow::{bail, Result};

pub(crate) fn selfhost_run_impl(opts: &SelfhostRunOptions) -> Result<()> {
    let output_name = format!(
        "{}.run.out",
        opts.input_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
    );

    let build_opts = SelfhostBuildOptions {
        input_file: opts.input_file.clone(),
        output_name: output_name.clone(),
        out_dir: opts.out_dir.clone(),
        check_only: false,
        release: opts.release,
    };

    println!("[1/2] building self-host executable ...");
    let output_path = selfhost_build_impl(&build_opts)?;

    println!("[2/2] running executable ...");
    let exe = resolve_exec(&output_path)?;
    let run_args: Vec<&str> = opts.program_args.iter().map(String::as_str).collect();
    let code = run_cmd_exit_code(&exe, &run_args, false)?;

    if let Some(expected) = opts.expect_exit {
        if code != expected {
            bail!("exit code mismatch, expected {}, got {}", expected, code);
        }
        println!("self-host run succeeded: expected exit code {}", expected);
        return Ok(());
    }

    println!("self-host run exit code: {}", code);
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}
