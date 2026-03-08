pub(crate) fn print_selfhost_build_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p xtask -- selfhost-build <input.lcy> [-o output] [--out-dir DIR] [--check-only] [--release]"
    );
}

pub(crate) fn print_selfhost_run_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p xtask -- selfhost-run <input.lcy> [--release] [--out-dir DIR] [--expect-exit N] [--] [program args...]"
    );
}
