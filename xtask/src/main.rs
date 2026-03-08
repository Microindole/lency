mod checks;
mod helpers;
mod selfhost;

use anyhow::{bail, Result};
use std::env;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_usage();
        bail!("missing xtask command");
    };

    let rest: Vec<String> = args.collect();
    match cmd.as_str() {
        "auto-check" => {
            ensure_no_args("auto-check", &rest)?;
            checks::auto_check()
        }
        "check-rust" => {
            ensure_no_args("check-rust", &rest)?;
            checks::check_rust()
        }
        "check-lency" => {
            ensure_no_args("check-lency", &rest)?;
            checks::check_lency()
        }
        "selfhost-build" => selfhost::selfhost_build_from_args(&rest),
        "selfhost-run" => selfhost::selfhost_run_from_args(&rest),
        _ => {
            print_usage();
            bail!("invalid xtask command: {cmd}")
        }
    }
}

fn ensure_no_args(cmd: &str, rest: &[String]) -> Result<()> {
    if !rest.is_empty() {
        bail!("{cmd} does not accept arguments");
    }
    Ok(())
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  cargo run -p xtask -- auto-check");
    eprintln!("  cargo run -p xtask -- check-rust");
    eprintln!("  cargo run -p xtask -- check-lency");
    eprintln!(
        "  cargo run -p xtask -- selfhost-build <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]"
    );
    eprintln!(
        "  cargo run -p xtask -- selfhost-run <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]"
    );
}
