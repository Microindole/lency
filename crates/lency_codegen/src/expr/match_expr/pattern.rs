use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::types::ToLLVMType;
use inkwell::values::PointerValue;
use lency_syntax::ast::{MatchPattern, Type};
use std::collections::HashMap;

/// Recursively generate pattern checks.
/// If any check fails, branch to `mismatch_bb`.
/// If successful, populates `bindings`.
/// Control flow falls through on success.
#[allow(clippy::only_used_in_recursion)]
pub(crate) fn gen_pattern_check<'ctx>(
    ctx: &CodegenContext<'ctx>,
    locals: &HashMap<String, (inkwell::values::PointerValue<'ctx>, lency_syntax::ast::Type)>,
    pattern: &MatchPattern,
    subject_ptr: PointerValue<'ctx>,
    subject_type: &Type,
    bindings: &mut Vec<(String, PointerValue<'ctx>, Type)>,
    mismatch_bb: inkwell::basic_block::BasicBlock<'ctx>,
) -> CodegenResult<()> {
    match pattern {
        MatchPattern::Wildcard => {
            // Always matches.
            Ok(())
        }
        MatchPattern::Variable(name) => {
            // Always matches, binds variable.
            // We just store the POINTER to the subject.
            // When user uses variable, they load from this pointer.
            bindings.push((name.clone(), subject_ptr, subject_type.clone()));
            Ok(())
        }
        MatchPattern::Literal(lit) => {
            // String Matching
            if matches!(subject_type, Type::String) {
                if let lency_syntax::ast::Literal::String(s) = lit {
                    // 1. Load Subject (i8**) -> i8*
                    let str_ptr = ctx
                        .builder
                        .build_load(
                            ctx.context
                                .i8_type()
                                .ptr_type(inkwell::AddressSpace::default()),
                            subject_ptr,
                            "str_load",
                        )
                        .unwrap()
                        .into_pointer_value();

                    // 2. Global String Literal
                    let lit_ptr = ctx
                        .builder
                        .build_global_string_ptr(s, "str_lit")
                        .map_err(|e| CodegenError::LLVMBuildError(e.to_string()))?
                        .as_pointer_value();

                    // 3. Strcmp
                    let strcmp_fn = ctx.module.get_function("strcmp").unwrap_or_else(|| {
                        let i32_type = ctx.context.i32_type();
                        let i8_ptr_type = ctx
                            .context
                            .i8_type()
                            .ptr_type(inkwell::AddressSpace::default());
                        let fn_type =
                            i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
                        ctx.module.add_function(
                            "strcmp",
                            fn_type,
                            Some(inkwell::module::Linkage::External),
                        )
                    });

                    let call = ctx
                        .builder
                        .build_call(
                            strcmp_fn,
                            &[str_ptr.into(), lit_ptr.into()],
                            "strcmp_result",
                        )
                        .unwrap();

                    let strcmp_result = call.try_as_basic_value().left().unwrap().into_int_value();
                    let zero = ctx.context.i32_type().const_int(0, false);
                    let cmp = ctx
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::EQ,
                            strcmp_result,
                            zero,
                            "streqtmp",
                        )
                        .unwrap();

                    let success_bb = ctx.context.append_basic_block(
                        ctx.builder
                            .get_insert_block()
                            .unwrap()
                            .get_parent()
                            .unwrap(),
                        "str_match_success",
                    );

                    ctx.builder
                        .build_conditional_branch(cmp, success_bb, mismatch_bb)
                        .unwrap();
                    ctx.builder.position_at_end(success_bb);
                    return Ok(());
                } else {
                    return Err(CodegenError::TypeMismatch);
                }
            }

            // Primitive Matching (Int/Bool)
            // Load value from pointer (unless subject is already loaded? No we standardized on ptr).
            // For primitive types (Int/Float/Bool), we load and compare.
            // String? -> Handled above.

            // 1. Load Subject
            // (Assuming int for now based on previous impl, but need to support others)

            let load_val = match subject_type {
                Type::Int | Type::Bool => ctx
                    .builder
                    .build_load(ctx.context.i64_type(), subject_ptr, "lit_chk_load")
                    .unwrap()
                    .into_int_value(),
                Type::Float => {
                    // Float equality is tricky?
                    // Use ordered equal `oeq`
                    let _fval = ctx
                        .builder
                        .build_load(ctx.context.f64_type(), subject_ptr, "lit_chk_fload")
                        .unwrap()
                        .into_float_value();
                    // We need to compare with literal.
                    // ...
                    return Err(CodegenError::UnsupportedFeature("Float matching".into()));
                }
                Type::String => unreachable!("String handled above"),
                _ => return Err(CodegenError::TypeMismatch),
            };

            let lit_val = match lit {
                lency_syntax::ast::Literal::Int(v) => {
                    ctx.context.i64_type().const_int(*v as u64, true)
                }
                lency_syntax::ast::Literal::Bool(b) => {
                    ctx.context.bool_type().const_int(*b as u64, false)
                }
                _ => {
                    return Err(CodegenError::UnsupportedFeature(
                        "Unsupported literal in match".into(),
                    ))
                }
            };

            // Note: Bool load would result in i64 or i1?
            // build_load type needs to match ptr type.
            // If Int, i64. If Bool, i1.
            // Adjust logic above.

            let cmp = if *subject_type == Type::Bool {
                let bload = ctx
                    .builder
                    .build_load(ctx.context.bool_type(), subject_ptr, "b_load")
                    .unwrap()
                    .into_int_value();
                ctx.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, bload, lit_val, "lit_eq")
                    .unwrap()
            } else {
                // Int
                ctx.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, load_val, lit_val, "lit_eq")
                    .unwrap()
            };

            let success_bb = ctx.context.append_basic_block(
                ctx.builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap(),
                "lit_match_success",
            );

            ctx.builder
                .build_conditional_branch(cmp, success_bb, mismatch_bb)
                .unwrap();
            ctx.builder.position_at_end(success_bb);
            Ok(())
        }
        MatchPattern::Variant {
            name: variant_name,
            sub_patterns,
        } => {
            // Enum Matching.
            // subject_ptr points to { i8, [size x i8] }

            // 1. Check Tag
            // We need to know the tag index for `variant_name`.
            // The Enum Name is in `subject_type`.
            let enum_name = match subject_type {
                Type::Struct(n) => n,
                Type::Generic(n, _) => n, // Generic Enum
                // Sprint 15: Treat Result<T, E> as enum "Result"
                Type::Result { .. } => "Result",
                _ => return Err(CodegenError::TypeMismatch),
            };

            // Look up variants info to find index
            // Sprint 15: Special handling for Result<T,E>
            let variants_info = if enum_name == "Result" {
                // Result.Ok and Result.Err are compiler built-ins
                // Ok has index 0 with one field of type T (GenericParam)
                // Err has index 1 with one field of type E (GenericParam)
                // We need to provide the variant info dynamically
                vec![
                    ("Ok".to_string(), vec![Type::GenericParam("T".to_string())]),
                    ("Err".to_string(), vec![Type::GenericParam("E".to_string())]),
                ]
            } else {
                ctx.enum_variants
                    .get(enum_name)
                    .ok_or(CodegenError::UndefinedStructType(enum_name.to_string()))?
                    .clone()
            };

            let (tag_idx, (_, field_types_ast)) = variants_info
                .iter()
                .enumerate()
                .find(|(_, (n, _))| n == variant_name)
                .ok_or(CodegenError::TypeMismatch)?; // Variant not found?

            // GEP Tag (element 0)
            // Sprint 15: Result struct type special handling
            let enum_struct_type = if enum_name == "Result" {
                // Result enum type is { i64 (tag), [max_size x i8] (payload) }
                // We need to ensure it exists in ctx.struct_types
                if let Some(st) = ctx.struct_types.get(enum_name) {
                    *st
                } else {
                    // Create Result struct type dynamically
                    // tag: i64, payload: arbitrary size array (use i64 for simplicity)
                    ctx.context.struct_type(
                        &[
                            ctx.context.i64_type().into(), // tag
                            ctx.context.i64_type().into(), // payload (simplified)
                        ],
                        false,
                    )
                }
            } else {
                *ctx.struct_types.get(enum_name).unwrap()
            };
            let tag_ptr = ctx
                .builder
                .build_struct_gep(enum_struct_type, subject_ptr, 0, "tag_ptr")
                .unwrap();

            let tag_val = ctx
                .builder
                .build_load(ctx.context.i64_type(), tag_ptr, "tag_val")
                .unwrap()
                .into_int_value();
            let expected_tag = ctx.context.i64_type().const_int(tag_idx as u64, false);

            let tag_cmp = ctx
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, tag_val, expected_tag, "tag_eq")
                .unwrap();

            let tag_success_bb = ctx.context.append_basic_block(
                ctx.builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap(),
                "tag_match_success",
            );
            ctx.builder
                .build_conditional_branch(tag_cmp, tag_success_bb, mismatch_bb)
                .unwrap();
            ctx.builder.position_at_end(tag_success_bb);

            // 2. Destructure Payload for Sub-patterns
            if !sub_patterns.is_empty() {
                // Bitcast payload (element 1) to variant struct layout
                let payload_arr_ptr = ctx
                    .builder
                    .build_struct_gep(enum_struct_type, subject_ptr, 1, "payload_arr")
                    .unwrap();

                // Construct Variant Body Type { field1, field2... }
                // We need LLVM types for fields.

                // Sprint 15: For Result<T,E>, substitute GenericParam with concrete types
                let field_types_concrete = if enum_name == "Result" && !field_types_ast.is_empty() {
                    // Extract concrete types from subject_type (Result<ok_type, err_type>)
                    match subject_type {
                        Type::Result { ok_type, err_type } => {
                            // Substitute T -> ok_type, E -> err_type
                            field_types_ast
                                .iter()
                                .map(|ty| match ty {
                                    Type::GenericParam(name) if name == "T" => (**ok_type).clone(),
                                    Type::GenericParam(name) if name == "E" => (**err_type).clone(),
                                    _ => ty.clone(),
                                })
                                .collect()
                        }
                        _ => field_types_ast.clone(),
                    }
                } else {
                    field_types_ast.clone()
                };

                let mut variant_llvm_types = Vec::new();
                for ty in &field_types_concrete {
                    variant_llvm_types.push(ty.to_llvm_type(ctx)?);
                }
                let variant_struct_type = ctx.context.struct_type(&variant_llvm_types, false);

                let payload_typed_ptr = ctx
                    .builder
                    .build_bitcast(
                        payload_arr_ptr,
                        variant_struct_type.ptr_type(inkwell::AddressSpace::default()),
                        "payload_typed",
                    )
                    .unwrap()
                    .into_pointer_value();

                // Recurse for each field
                for (i, sub_pat) in sub_patterns.iter().enumerate() {
                    // GEP Field i
                    let field_ptr = ctx
                        .builder
                        .build_struct_gep(
                            variant_struct_type,
                            payload_typed_ptr,
                            i as u32,
                            "field_ptr",
                        )
                        .unwrap();
                    let field_type = &field_types_concrete[i]; // Use concrete types with GenericParam replaced

                    gen_pattern_check(
                        ctx,
                        locals,
                        sub_pat,
                        field_ptr,
                        field_type,
                        bindings,
                        mismatch_bb,
                    )?;
                }
            }

            Ok(())
        }
    }
}
