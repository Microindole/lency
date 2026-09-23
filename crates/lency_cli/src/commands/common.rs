use crate::lir_backend;
use anyhow::{bail, Context, Result};
use lency_driver::compile_file;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::PathBuf};

pub fn compile_to_llvm_ir(input: &str) -> Result<String> {
    if input.ends_with(".lir") {
        let source = fs::read_to_string(input)?;
        return lir_backend::compile_lir_to_llvm_ir(&source);
    }
    Ok(compile_file(input)?.ir)
}

pub fn find_runtime_dir() -> Option<PathBuf> {
    let libs = [
        "liblency_runtime.so",
        "liblency_runtime.dylib",
        "liblency_runtime.a",
        "lency_runtime.dll.lib",
        "lency_runtime.lib",
        "lency_runtime.dll",
    ];

    for dir in runtime_search_dirs() {
        for lib in libs {
            let path = dir.join(lib);
            if path.exists() {
                return Some(dir);
            }
        }
    }
    None
}

pub fn temp_artifact_path(ext: &str) -> Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::current_dir().context("failed to get project root")?;
    let tmp_dir = root.join("tmp");
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create temp dir: {}", tmp_dir.display()))?;
    Ok(tmp_dir.join(format!("lency_{}_{}.{}", std::process::id(), ts, ext)))
}

pub fn require_tool(candidates: &[&str], tool_desc: &str) -> Result<PathBuf> {
    if let Some(path) = find_tool(candidates) {
        return Ok(path);
    }

    let joined = candidates.join(", ");
    let llvm_prefix_hint = std::env::var("LLVM_SYS_150_PREFIX").ok();
    if let Some(prefix) = llvm_prefix_hint {
        bail!(
            "required tool not found ({tool_desc}): tried [{joined}] in LLVM_SYS_150_PREFIX/bin and PATH (LLVM_SYS_150_PREFIX={prefix})"
        );
    }

    bail!(
        "required tool not found ({tool_desc}): tried [{joined}] in LLVM_SYS_150_PREFIX/bin and PATH"
    )
}

fn find_tool(candidates: &[&str]) -> Option<PathBuf> {
    if let Ok(prefix) = std::env::var("LLVM_SYS_150_PREFIX") {
        let bin = PathBuf::from(prefix).join("bin");
        for name in with_platform_names(candidates) {
            let p = bin.join(&name);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        for name in with_platform_names(candidates) {
            let p = dir.join(&name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

pub fn find_runtime_library() -> Option<PathBuf> {
    let libs: &[&str] = if cfg!(windows) {
        &[
            "lency_runtime.dll.lib",
            "lency_runtime.lib",
            "liblency_runtime.a",
        ]
    } else if cfg!(target_os = "macos") {
        &["liblency_runtime.dylib", "liblency_runtime.a"]
    } else {
        &["liblency_runtime.so", "liblency_runtime.a"]
    };

    for dir in runtime_search_dirs() {
        for lib in libs {
            let path = dir.join(lib);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

fn runtime_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(executable) = std::env::current_exe() {
        if let Some(parent) = executable.parent() {
            dirs.push(parent.to_path_buf());
            dirs.push(parent.join("lib"));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("target/release"));
        dirs.push(cwd.join("target/debug"));
    }

    dirs.dedup();
    dirs
}

fn with_platform_names(candidates: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for c in candidates {
        out.push((*c).to_string());
        if cfg!(windows) && !c.to_ascii_lowercase().ends_with(".exe") {
            out.push(format!("{c}.exe"));
        }
    }
    out
}
