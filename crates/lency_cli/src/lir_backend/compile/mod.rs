mod call;
mod helpers;
mod member_call;

use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

use super::emitter::{Emitter, ValueType};
use call::emit_call_assignment;
use helpers::{build_output_ir, collect_vars};

/// Compile LIR text emitted by lencyc `--emit-lir` into LLVM IR.
pub fn compile_lir_to_llvm_ir(source: &str) -> Result<String> {
    let vars = collect_vars(source)?;
    let mut emitter = Emitter::new(vars.clone());
    let mut member_call_targets: HashMap<String, (String, ValueType, String)> = HashMap::new();

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
                emit_call_assignment(&mut emitter, dst, rest, &member_call_targets)?;
                continue;
            }

            if let Some(rest) = rhs.strip_prefix("get ") {
                let (obj_raw, member_name) = rest
                    .split_once('.')
                    .ok_or_else(|| anyhow!("invalid get instruction: {}", line))?;
                let obj_name = obj_raw.trim();
                let member_name = member_name.trim();
                let (obj_repr, obj_ty) = emitter.emit_operand(obj_name)?;
                // Generic member reference placeholder for subsequent `call %tX(...)`.
                emitter.push(format!("  {} = inttoptr i64 0 to ptr", dst));
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

    Ok(build_output_ir(emitter))
}
