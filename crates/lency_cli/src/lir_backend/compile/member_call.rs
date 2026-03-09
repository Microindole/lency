use anyhow::{bail, Result};

use super::super::emitter::{llvm_type_str, Emitter, ValueType};

fn guess_member_call_return_type(member_name: &str) -> ValueType {
    match member_name {
        "to_string" | "trim" | "substr" | "replace_first" | "replace_all" | "repeat"
        | "pad_right" | "pad_left" | "to_upper" | "to_lower" | "reverse" | "trim_left"
        | "trim_right" | "join" | "format" | "get_extension" | "get_filename" | "get_directory"
        | "join_path" => ValueType::Ptr,
        "contains" | "starts_with" | "ends_with" | "is_empty" | "is_alpha" | "is_digit"
        | "is_alphanumeric" | "is_whitespace" | "is_hex_digit" | "is_printable"
        | "is_punctuation" | "is_lower" | "is_upper" | "is_close" => ValueType::I1,
        _ => ValueType::I64,
    }
}

fn resolve_member_intrinsic_call(
    member_name: &str,
    call_argc: usize,
) -> Result<Option<(&'static str, Vec<ValueType>, ValueType)>> {
    let sig = match member_name {
        "to_string" => Some((
            "lency_int_to_string",
            vec![ValueType::I64],
            ValueType::Ptr,
            0usize,
        )),
        "len" => Some((
            "lency_string_len",
            vec![ValueType::Ptr],
            ValueType::I64,
            0usize,
        )),
        "trim" => Some((
            "lency_string_trim",
            vec![ValueType::Ptr],
            ValueType::Ptr,
            0usize,
        )),
        "substr" => Some((
            "lency_string_substr",
            vec![ValueType::Ptr, ValueType::I64, ValueType::I64],
            ValueType::Ptr,
            2usize,
        )),
        "split" => Some((
            "lency_string_split",
            vec![ValueType::Ptr, ValueType::Ptr],
            ValueType::Ptr,
            1usize,
        )),
        "format" => Some((
            "lency_string_format",
            vec![ValueType::Ptr, ValueType::Ptr],
            ValueType::Ptr,
            1usize,
        )),
        "join" => Some((
            "lency_string_join",
            vec![ValueType::Ptr, ValueType::Ptr],
            ValueType::Ptr,
            1usize,
        )),
        _ => None,
    };

    if let Some((runtime_name, arg_tys, ret_ty, expected_call_argc)) = sig {
        if call_argc != expected_call_argc {
            bail!(
                "invalid call arity for member '{}': expected {}, got {}",
                member_name,
                expected_call_argc,
                call_argc
            );
        }
        return Ok(Some((runtime_name, arg_tys, ret_ty)));
    }

    Ok(None)
}

pub(super) fn emit_member_call(
    emitter: &mut Emitter,
    dst: &str,
    obj_repr: String,
    obj_ty: ValueType,
    member_name: &str,
    parsed_args: &[&str],
) -> Result<()> {
    if let Some((runtime_name, intrinsic_arg_tys, intrinsic_ret_ty)) =
        resolve_member_intrinsic_call(member_name, parsed_args.len())?
    {
        let mut intrinsic_arg_values: Vec<(String, ValueType)> = Vec::new();
        intrinsic_arg_values.push((obj_repr, obj_ty));
        for arg in parsed_args {
            let (arg_repr, arg_ty) = emitter.emit_operand(arg.trim())?;
            intrinsic_arg_values.push((arg_repr, arg_ty));
        }

        if intrinsic_arg_values.len() != intrinsic_arg_tys.len() {
            bail!(
                "invalid intrinsic member lowering for '{}': expected {} args, got {}",
                member_name,
                intrinsic_arg_tys.len(),
                intrinsic_arg_values.len()
            );
        }

        let mut casted_values: Vec<(String, ValueType)> = Vec::new();
        for (idx, (arg_repr, arg_ty)) in intrinsic_arg_values.iter().enumerate() {
            let (casted, casted_ty) =
                emitter.cast_to_type(arg_repr.clone(), *arg_ty, intrinsic_arg_tys[idx]);
            casted_values.push((casted, casted_ty));
        }

        emitter.note_extern_func(runtime_name, intrinsic_arg_tys.clone(), intrinsic_ret_ty)?;
        let args_sig = casted_values
            .iter()
            .map(|(repr, ty)| format!("{} {}", llvm_type_str(*ty), repr))
            .collect::<Vec<_>>()
            .join(", ");
        emitter.push(format!(
            "  {} = call {} @{}({})",
            dst,
            llvm_type_str(intrinsic_ret_ty),
            runtime_name,
            args_sig
        ));
        emitter.mark_temp(dst, intrinsic_ret_ty);
        return Ok(());
    }

    // Generic member-call fallback: `obj.member(a, b)` => `member(obj, a, b)`.
    let mut arg_values: Vec<(String, ValueType)> = Vec::new();
    arg_values.push((obj_repr, obj_ty));
    for arg in parsed_args {
        let (arg_repr, arg_ty) = emitter.emit_operand(arg.trim())?;
        arg_values.push((arg_repr, arg_ty));
    }
    let arg_tys = arg_values.iter().map(|(_, ty)| *ty).collect::<Vec<_>>();
    let ret_ty = guess_member_call_return_type(member_name);
    emitter.note_extern_func(member_name, arg_tys, ret_ty)?;
    let args_sig = arg_values
        .iter()
        .map(|(repr, ty)| format!("{} {}", llvm_type_str(*ty), repr))
        .collect::<Vec<_>>()
        .join(", ");
    emitter.push(format!(
        "  {} = call {} @{}({})",
        dst,
        llvm_type_str(ret_ty),
        member_name,
        args_sig
    ));
    emitter.mark_temp(dst, ret_ty);
    Ok(())
}
