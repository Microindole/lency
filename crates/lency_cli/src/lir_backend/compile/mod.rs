mod call;
mod helpers;
mod member_call;

use anyhow::{anyhow, bail, Result};
use std::collections::{HashMap, HashSet};

use super::emitter::{Emitter, ValueType};
use call::{emit_call_assignment, emit_call_statement};
use helpers::{
    build_function_signatures, build_output_ir, collect_vars, llvm_function_ret_ty,
    parse_functions, split_top_level_commas, LirFunction,
};

fn compile_function(
    func: &LirFunction,
    function_sigs: &HashMap<String, (Vec<ValueType>, ValueType)>,
    string_global_offset: usize,
) -> Result<Emitter> {
    let mut vars = collect_vars(&func.body_lines)?;
    for (param_name, _) in &func.params {
        vars.insert(param_name.clone());
    }
    let mut emitter = Emitter::new(vars.clone());
    emitter.set_string_global_offset(string_global_offset);
    let mut member_call_targets: HashMap<String, (String, ValueType, String)> = HashMap::new();

    let header_ret_ty = llvm_function_ret_ty(&func.name, func.ret_ty);
    let header_params = if func.name == "main" {
        "i32 %process_argc, i8** %process_argv".to_string()
    } else {
        func.params
            .iter()
            .map(|(name, ty)| format!("{} {}", super::emitter::llvm_type_str(*ty), name))
            .collect::<Vec<_>>()
            .join(", ")
    };
    emitter.push(format!(
        "define {} @{}({}) {{",
        header_ret_ty, func.name, header_params
    ));
    emitter.push("entry:");
    if func.name == "main" {
        emitter.push("  store i32 %process_argc, i32* @lency_process_argc");
        emitter.push("  store i8** %process_argv, i8*** @lency_process_argv");
    }

    for var in &vars {
        emitter.push(format!("  {}.addr = alloca i64", var));
    }
    for (param_name, param_ty) in &func.params {
        let (stored_repr, _) = emitter.ensure_i64(param_name.clone(), *param_ty);
        emitter.push(format!(
            "  store i64 {}, i64* {}.addr",
            stored_repr, param_name
        ));
    }

    for raw in &func.body_lines {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
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

        if let Some(rest) = line.strip_prefix("call ") {
            emit_call_statement(&mut emitter, rest, function_sigs)?;
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
            if func.name == "main" {
                emitter.push("  ret i32 0");
            } else {
                emitter.push("  ret void");
            }
            emitter.terminated = true;
            continue;
        }

        if line.starts_with("ret ") {
            let val = line.trim_start_matches("ret ").trim();
            let (repr, ty) = emitter.emit_operand(val)?;
            let target_ty = if func.name == "main" {
                ValueType::I64
            } else {
                func.ret_ty
            };
            match target_ty {
                ValueType::I64 => {
                    let (repr_i64, _) = emitter.ensure_i64(repr, ty);
                    if func.name == "main" {
                        let code = emitter.next_tmp("ret_i32");
                        emitter.push(format!("  {} = trunc i64 {} to i32", code, repr_i64));
                        emitter.push(format!("  ret i32 {}", code));
                    } else {
                        emitter.push(format!("  ret i64 {}", repr_i64));
                    }
                }
                ValueType::I1 => {
                    let (repr_i1, _) = emitter.ensure_i1(repr, ty);
                    emitter.push(format!("  ret i1 {}", repr_i1));
                }
                ValueType::Ptr => {
                    let (repr_ptr, _) = emitter.ensure_ptr(repr, ty);
                    if func.name == "main" {
                        let widened = emitter.next_tmp("ptrtoint");
                        emitter.push(format!("  {} = ptrtoint i8* {} to i64", widened, repr_ptr));
                        let code = emitter.next_tmp("ret_i32");
                        emitter.push(format!("  {} = trunc i64 {} to i32", code, widened));
                        emitter.push(format!("  ret i32 {}", code));
                    } else {
                        emitter.push(format!("  ret i8* {}", repr_ptr));
                    }
                }
                ValueType::Void => {
                    bail!("cannot return value from void function '{}'", func.name);
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
                bail!(
                    "unsupported expr_unknown placeholder in function '{}': {}",
                    func.name,
                    line
                );
            }

            if let Some(rest) = rhs.strip_prefix("call ") {
                emit_call_assignment(&mut emitter, dst, rest, &member_call_targets, function_sigs)?;
                continue;
            }

            if let Some(rest) = rhs.strip_prefix("get ") {
                let (obj_raw, member_name) = rest
                    .split_once('.')
                    .ok_or_else(|| anyhow!("invalid get instruction: {}", line))?;
                let obj_name = obj_raw.trim();
                let member_name = member_name.trim();
                let (obj_repr, obj_ty) = emitter.emit_operand(obj_name)?;
                emitter.push(format!("  {} = inttoptr i64 0 to i8*", dst));
                emitter.mark_temp(dst, ValueType::Ptr);
                member_call_targets
                    .insert(dst.to_string(), (obj_repr, obj_ty, member_name.to_string()));
                continue;
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
                let operands = split_top_level_commas(rhs_joined);
                if operands.len() != 2 {
                    bail!("invalid binary operands: {}", line);
                }
                let lhs = operands[0];
                let rhs_val = operands[1];
                emitter.emit_binary(dst, op, lhs.trim(), rhs_val.trim())?;
                continue;
            }

            bail!("unsupported assignment form: {}", line);
        }

        bail!("unsupported lir line: {}", line);
    }

    if !emitter.terminated {
        if func.name == "main" {
            emitter.push("  ret i32 0");
        } else {
            match func.ret_ty {
                ValueType::Void => emitter.push("  ret void"),
                ValueType::I64 => emitter.push("  ret i64 0"),
                ValueType::I1 => emitter.push("  ret i1 false"),
                ValueType::Ptr => emitter.push("  ret i8* null"),
            }
        }
    }
    emitter.push("}");
    Ok(emitter)
}

/// Compile LIR text emitted by lencyc `--emit-lir` into LLVM IR.
pub fn compile_lir_to_llvm_ir(source: &str) -> Result<String> {
    let functions = parse_functions(source)?;
    let function_sigs = build_function_signatures(&functions);
    let mut all_lines = Vec::new();
    let mut all_string_globals = HashMap::new();
    let mut all_extern_funcs = HashMap::new();
    let mut string_global_offset = 0;

    for func in &functions {
        let emitter = compile_function(func, &function_sigs, string_global_offset)?;
        let Emitter {
            lines,
            string_globals,
            extern_funcs,
            ..
        } = emitter;
        string_global_offset += string_globals.len();
        all_lines.extend(lines);
        for (literal, tuple) in string_globals {
            let unique_key = format!("{}\0{}", literal, tuple.0);
            all_string_globals.insert(unique_key, tuple);
        }
        for (name, sig) in extern_funcs {
            all_extern_funcs.entry(name).or_insert(sig);
        }
    }

    let mut emitter = Emitter::new(HashSet::new());
    emitter.lines = all_lines;
    emitter.string_globals = all_string_globals;
    emitter.extern_funcs = all_extern_funcs;

    Ok(build_output_ir(emitter))
}
