use anyhow::{anyhow, Result};
use std::collections::HashSet;

use super::super::emitter::{llvm_type_str, Emitter, ExternSig, ValueType};

pub(super) fn collect_vars(source: &str) -> Result<HashSet<String>> {
    let mut vars = HashSet::new();
    for raw in source.lines() {
        let line = raw.trim();
        if line.starts_with("var %") {
            let rest = line
                .strip_prefix("var ")
                .ok_or_else(|| anyhow!("invalid var line: {}", line))?;
            let (name, _) = rest
                .split_once(" = ")
                .ok_or_else(|| anyhow!("invalid var assignment: {}", line))?;
            vars.insert(name.trim().to_string());
        } else if line.starts_with("store %") {
            let rest = line
                .strip_prefix("store ")
                .ok_or_else(|| anyhow!("invalid store line: {}", line))?;
            let (name, _) = rest
                .split_once(", ")
                .ok_or_else(|| anyhow!("invalid store assignment: {}", line))?;
            vars.insert(name.trim().to_string());
        }
    }
    Ok(vars)
}

pub(super) fn resolve_builtin_call(
    callee_name: &str,
) -> Option<(&'static str, Vec<ValueType>, ValueType)> {
    // 当前仅映射 runtime ABI 已稳定的 builtin 子集。
    match callee_name {
        "arg_count" => Some(("lency_arg_count", vec![], ValueType::I64)),
        "arg_at" => Some(("lency_arg_at", vec![ValueType::I64], ValueType::Ptr)),
        "int_to_string" => Some(("lency_int_to_string", vec![ValueType::I64], ValueType::Ptr)),
        "file_exists" => Some(("lency_file_exists", vec![ValueType::Ptr], ValueType::I64)),
        "is_dir" => Some(("lency_file_is_dir", vec![ValueType::Ptr], ValueType::I64)),
        _ => None,
    }
}

pub(super) fn build_output_ir(emitter: Emitter) -> String {
    let mut out_lines = Vec::new();
    let mut extern_names = emitter.extern_funcs.keys().cloned().collect::<Vec<_>>();
    extern_names.sort();

    for name in extern_names {
        let sig = emitter
            .extern_funcs
            .get(&name)
            .cloned()
            .unwrap_or(ExternSig {
                arg_tys: vec![],
                ret_ty: ValueType::I64,
            });
        if sig.arg_tys.is_empty() {
            out_lines.push(format!("declare {} @{}()", llvm_type_str(sig.ret_ty), name));
        } else {
            let args_sig = sig
                .arg_tys
                .iter()
                .map(|ty| llvm_type_str(*ty).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            out_lines.push(format!(
                "declare {} @{}({})",
                llvm_type_str(sig.ret_ty),
                name,
                args_sig
            ));
        }
    }
    if !out_lines.is_empty() {
        out_lines.push(String::new());
    }
    out_lines.extend(emitter.lines);

    format!("{}\n", out_lines.join("\n"))
}
