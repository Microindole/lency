use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

use super::super::emitter::{llvm_type_str, Emitter, ValueType};
use super::helpers::resolve_builtin_call;
use super::member_call::emit_member_call;

pub(super) fn emit_call_assignment(
    emitter: &mut Emitter,
    dst: &str,
    rest: &str,
    member_call_targets: &HashMap<String, (String, ValueType, String)>,
) -> Result<()> {
    if rest == "?()" {
        emitter.push(format!("  {} = add i64 0, 0", dst));
        emitter.mark_temp(dst, ValueType::I64);
        return Ok(());
    }

    let (callee, args_raw) = rest
        .split_once('(')
        .ok_or_else(|| anyhow!("invalid call instruction: call {}", rest))?;
    let args_raw = args_raw
        .strip_suffix(')')
        .ok_or_else(|| anyhow!("invalid call instruction: call {}", rest))?;
    let parsed_args = if args_raw.trim().is_empty() {
        vec![]
    } else {
        args_raw.trim().split(", ").collect::<Vec<_>>()
    };

    let callee = callee.trim();
    if !callee.starts_with('%') {
        bail!("unsupported call callee: {}", callee);
    }

    if let Some((obj_repr, obj_ty, member_name)) = member_call_targets.get(callee).cloned() {
        return emit_member_call(emitter, dst, obj_repr, obj_ty, &member_name, &parsed_args);
    }

    if parsed_args.is_empty() {
        if let Ok((callee_value, callee_ty)) = emitter.emit_operand(callee) {
            match callee_ty {
                ValueType::I64 => {
                    emitter.push(format!("  {} = add i64 {}, 0", dst, callee_value));
                    emitter.mark_temp(dst, ValueType::I64);
                    return Ok(());
                }
                ValueType::I1 => {
                    emitter.push(format!("  {} = xor i1 {}, false", dst, callee_value));
                    emitter.mark_temp(dst, ValueType::I1);
                    return Ok(());
                }
                ValueType::Ptr => {
                    emitter.push(format!(
                        "  {} = getelementptr i8, ptr {}, i64 0",
                        dst, callee_value
                    ));
                    emitter.mark_temp(dst, ValueType::Ptr);
                    return Ok(());
                }
            }
        }
    }

    let callee_name = callee.trim_start_matches('%');
    if callee_name.is_empty() {
        bail!("invalid call callee: call {}", rest);
    }

    let default_arg_tys = if args_raw.trim().is_empty() {
        vec![]
    } else {
        args_raw
            .trim()
            .split(", ")
            .map(|_| ValueType::I64)
            .collect()
    };
    let (llvm_callee_name, arg_tys, ret_ty) =
        if let Some((builtin_name, builtin_arg_tys, builtin_ret_ty)) =
            resolve_builtin_call(callee_name)
        {
            (builtin_name, builtin_arg_tys, builtin_ret_ty)
        } else {
            (callee_name, default_arg_tys, ValueType::I64)
        };

    if parsed_args.len() != arg_tys.len() {
        bail!(
            "invalid call arity for '{}': expected {}, got {}",
            callee_name,
            arg_tys.len(),
            parsed_args.len()
        );
    }

    let mut arg_values: Vec<(String, ValueType)> = Vec::new();
    for (idx, arg) in parsed_args.iter().enumerate() {
        let (arg_repr, arg_ty) = emitter.emit_operand(arg.trim())?;
        let (casted, casted_ty) = emitter.cast_to_type(arg_repr, arg_ty, arg_tys[idx]);
        arg_values.push((casted, casted_ty));
    }

    emitter.note_extern_func(llvm_callee_name, arg_tys.clone(), ret_ty)?;
    let args_sig = arg_values
        .iter()
        .map(|(repr, ty)| format!("{} {}", llvm_type_str(*ty), repr))
        .collect::<Vec<_>>()
        .join(", ");
    emitter.push(format!(
        "  {} = call {} @{}({})",
        dst,
        llvm_type_str(ret_ty),
        llvm_callee_name,
        args_sig
    ));
    emitter.mark_temp(dst, ret_ty);
    Ok(())
}
