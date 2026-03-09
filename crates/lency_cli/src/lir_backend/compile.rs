use anyhow::{anyhow, bail, Result};
use std::collections::HashSet;

use super::emitter::{llvm_type_str, Emitter, ExternSig, ValueType};

fn collect_vars(source: &str) -> Result<HashSet<String>> {
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

fn resolve_builtin_call(callee_name: &str) -> Option<(&'static str, Vec<ValueType>, ValueType)> {
    // FIXME: 这里仅覆盖“早期 Lency 自举最小可用”的 builtin 子集。
    // `read_to_string` / `write_string` / `parse_float` 等需要 richer ABI
    // （Result/float/多返回值）后，才能在 LIR 最小后端完整支持。
    match callee_name {
        "arg_count" => Some(("lency_arg_count", vec![], ValueType::I64)),
        "arg_at" => Some(("lency_arg_at", vec![ValueType::I64], ValueType::Ptr)),
        "int_to_string" => Some(("lency_int_to_string", vec![ValueType::I64], ValueType::Ptr)),
        "file_exists" => Some(("lency_file_exists", vec![ValueType::Ptr], ValueType::I64)),
        "is_dir" => Some(("lency_file_is_dir", vec![ValueType::Ptr], ValueType::I64)),
        _ => None,
    }
}

/// Compile LIR text emitted by lencyc `--emit-lir` into LLVM IR.
pub fn compile_lir_to_llvm_ir(source: &str) -> Result<String> {
    let vars = collect_vars(source)?;
    let mut emitter = Emitter::new(vars.clone());

    emitter.push("define i32 @main() {");
    emitter.push("entry:");

    for var in &vars {
        emitter.push(format!("  {}.addr = alloca i64", var));
    }

    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line == "func main {" || line == "}" {
            continue;
        }
        if line.ends_with(':') {
            emitter.terminated = false;
            if line == "entry:" {
                continue;
            }
            emitter.push(line.to_string());
            continue;
        }
        if emitter.terminated {
            continue;
        }

        if line.starts_with("var %") {
            let rest = line
                .strip_prefix("var ")
                .ok_or_else(|| anyhow!("invalid var line: {}", line))?;
            let (name, rhs) = rest
                .split_once(" = ")
                .ok_or_else(|| anyhow!("invalid var line: {}", line))?;
            let (rhs_repr, rhs_ty) = emitter.emit_operand(rhs.trim())?;
            emitter.emit_store_var(name.trim(), rhs_repr, rhs_ty)?;
            continue;
        }

        if line.starts_with("store %") {
            let rest = line
                .strip_prefix("store ")
                .ok_or_else(|| anyhow!("invalid store line: {}", line))?;
            let (name, rhs) = rest
                .split_once(", ")
                .ok_or_else(|| anyhow!("invalid store line: {}", line))?;
            let (rhs_repr, rhs_ty) = emitter.emit_operand(rhs.trim())?;
            emitter.emit_store_var(name.trim(), rhs_repr, rhs_ty)?;
            continue;
        }

        if line.starts_with("jmp ") {
            let label = line.trim_start_matches("jmp ").trim();
            emitter.push(format!("  br label %{}", label));
            emitter.terminated = true;
            continue;
        }

        if line.starts_with("br ") {
            let rest = line
                .trim_start_matches("br ")
                .trim()
                .split(", ")
                .collect::<Vec<_>>();
            if rest.len() != 3 {
                bail!("invalid br instruction: {}", line);
            }
            let (cond_repr, cond_ty) = emitter.emit_operand(rest[0].trim())?;
            let (cond_i1, _) = emitter.ensure_i1(cond_repr, cond_ty);
            emitter.push(format!(
                "  br i1 {}, label %{}, label %{}",
                cond_i1,
                rest[1].trim(),
                rest[2].trim()
            ));
            emitter.terminated = true;
            continue;
        }

        if line == "ret" {
            emitter.push("  ret i32 0");
            emitter.terminated = true;
            continue;
        }

        if line.starts_with("ret ") {
            let val = line.trim_start_matches("ret ").trim();
            let (repr, ty) = emitter.emit_operand(val)?;
            match ty {
                ValueType::I64 => {
                    let code = emitter.next_tmp("ret_i32");
                    emitter.push(format!("  {} = trunc i64 {} to i32", code, repr));
                    emitter.push(format!("  ret i32 {}", code));
                }
                ValueType::I1 => {
                    let code = emitter.next_tmp("ret_i32");
                    emitter.push(format!("  {} = zext i1 {} to i32", code, repr));
                    emitter.push(format!("  ret i32 {}", code));
                }
                ValueType::Ptr => {
                    let widened = emitter.next_tmp("ptrtoint");
                    emitter.push(format!("  {} = ptrtoint ptr {} to i64", widened, repr));
                    let code = emitter.next_tmp("ret_i32");
                    emitter.push(format!("  {} = trunc i64 {} to i32", code, widened));
                    emitter.push(format!("  ret i32 {}", code));
                }
            }
            emitter.terminated = true;
            continue;
        }

        if line.starts_with('%') && line.contains(" = ") {
            let (dst, rhs) = line
                .split_once(" = ")
                .ok_or_else(|| anyhow!("invalid assignment: {}", line))?;
            let dst = dst.trim();
            if rhs == "expr_unknown" {
                emitter.push(format!("  {} = add i64 0, 0", dst));
                emitter.mark_temp(dst, ValueType::I64);
                continue;
            }

            if let Some(rest) = rhs.strip_prefix("call ") {
                if rest == "?()" {
                    emitter.push(format!("  {} = add i64 0, 0", dst));
                    emitter.mark_temp(dst, ValueType::I64);
                    continue;
                }
                let (callee, args_raw) = rest
                    .split_once('(')
                    .ok_or_else(|| anyhow!("invalid call instruction: {}", line))?;
                let args_raw = args_raw
                    .strip_suffix(')')
                    .ok_or_else(|| anyhow!("invalid call instruction: {}", line))?;
                let parsed_args = if args_raw.trim().is_empty() {
                    vec![]
                } else {
                    args_raw.trim().split(", ").collect::<Vec<_>>()
                };

                let callee = callee.trim();
                if !callee.starts_with('%') {
                    // FIXME: LIR call lowering currently only supports `%symbol(...)`.
                    bail!("unsupported call callee: {}", callee);
                }

                if parsed_args.is_empty() {
                    if let Ok((callee_value, callee_ty)) = emitter.emit_operand(callee) {
                        match callee_ty {
                            ValueType::I64 => {
                                emitter.push(format!("  {} = add i64 {}, 0", dst, callee_value));
                                emitter.mark_temp(dst, ValueType::I64);
                                continue;
                            }
                            ValueType::I1 => {
                                emitter.push(format!("  {} = xor i1 {}, false", dst, callee_value));
                                emitter.mark_temp(dst, ValueType::I1);
                                continue;
                            }
                            ValueType::Ptr => {
                                emitter.push(format!(
                                    "  {} = getelementptr i8, ptr {}, i64 0",
                                    dst, callee_value
                                ));
                                emitter.mark_temp(dst, ValueType::Ptr);
                                continue;
                            }
                        }
                    }
                }

                let callee_name = callee.trim_start_matches('%');
                if callee_name.is_empty() {
                    bail!("invalid call callee: {}", line);
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
                continue;
            }

            if let Some(rest) = rhs.strip_prefix("get ") {
                let (obj_raw, member_name) = rest
                    .split_once('.')
                    .ok_or_else(|| anyhow!("invalid get instruction: {}", line))?;
                let obj_name = obj_raw.trim();
                let member_name = member_name.trim();
                let (obj_repr, obj_ty) = emitter.emit_operand(obj_name)?;
                if member_name == "to_string" {
                    let (arg_repr, _) = emitter.ensure_i64(obj_repr, obj_ty);
                    emitter.note_extern_func(
                        "lency_int_to_string",
                        vec![ValueType::I64],
                        ValueType::Ptr,
                    )?;
                    emitter.push(format!(
                        "  {} = call ptr @lency_int_to_string(i64 {})",
                        dst, arg_repr
                    ));
                    emitter.mark_temp(dst, ValueType::Ptr);
                    continue;
                }
                if member_name == "len" {
                    let (arg_repr, _) = emitter.ensure_ptr(obj_repr, obj_ty);
                    emitter.note_extern_func(
                        "lency_string_len",
                        vec![ValueType::Ptr],
                        ValueType::I64,
                    )?;
                    emitter.push(format!(
                        "  {} = call i64 @lency_string_len(ptr {})",
                        dst, arg_repr
                    ));
                    emitter.mark_temp(dst, ValueType::I64);
                    continue;
                }
                if member_name == "trim" {
                    let (arg_repr, _) = emitter.ensure_ptr(obj_repr, obj_ty);
                    emitter.note_extern_func(
                        "lency_string_trim",
                        vec![ValueType::Ptr],
                        ValueType::Ptr,
                    )?;
                    emitter.push(format!(
                        "  {} = call ptr @lency_string_trim(ptr {})",
                        dst, arg_repr
                    ));
                    emitter.mark_temp(dst, ValueType::Ptr);
                    continue;
                }
                // FIXME: 非 to_string/len/trim 的成员访问 lowering 仍未实现。
                bail!("unsupported get member in minimal LIR backend: {}", line);
            }

            let parts = rhs.split_whitespace().collect::<Vec<_>>();
            if parts.len() == 2 {
                emitter.emit_unary(dst, parts[0], parts[1])?;
                continue;
            }
            if parts.len() >= 3 {
                let op = parts[0];
                let rhs_joined = rhs
                    .strip_prefix(op)
                    .ok_or_else(|| anyhow!("invalid binary instruction: {}", line))?
                    .trim();
                let (lhs, rhs_val) = rhs_joined
                    .split_once(", ")
                    .ok_or_else(|| anyhow!("invalid binary operands: {}", line))?;
                emitter.emit_binary(dst, op, lhs.trim(), rhs_val.trim())?;
                continue;
            }

            bail!("unsupported assignment form: {}", line);
        }

        bail!("unsupported lir line: {}", line);
    }

    if !emitter.terminated {
        emitter.push("  ret i32 0");
    }
    emitter.push("}");

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

    Ok(format!("{}\n", out_lines.join("\n")))
}
