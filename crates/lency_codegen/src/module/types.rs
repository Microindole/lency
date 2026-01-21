use super::ModuleGenerator;
use crate::error::CodegenResult;
use crate::types::ToLLVMType;
use inkwell::targets::TargetData;
use inkwell::types::BasicType;
use lency_syntax::ast::{Decl, EnumVariant, Program, Type};

impl<'ctx, 'a> ModuleGenerator<'ctx, 'a> {
    /// 注册 Result<T, E> 类型到 struct_types
    pub(crate) fn register_result_type(&mut self, result_ty: &Type) -> CodegenResult<()> {
        if let Type::Result { ok_type, err_type } = result_ty {
            let mangled_name = lency_monomorph::mangling::mangle_type(result_ty);

            // 如果已注册，直接返回
            if self.ctx.struct_types.contains_key(&mangled_name) {
                return Ok(());
            }

            // 检查 err_type 是否已注册（如果是 Struct 类型）
            if let Type::Struct(err_name) = &**err_type {
                if !self.ctx.struct_types.contains_key(err_name) {
                    // Error struct 不存在，跳过注册（可能是单元测试）
                    return Ok(());
                }
            }

            // 创建字段类型
            let mut field_types = vec![
                self.ctx.context.bool_type().into(), // is_ok flag
            ];

            if !matches!(**ok_type, Type::Void) {
                field_types.push(ok_type.to_llvm_type(&*self.ctx)?);
            }
            if !matches!(**err_type, Type::Void) {
                field_types.push(err_type.to_llvm_type(&*self.ctx)?);
            }

            // 创建并注册结构体
            let struct_type = self.ctx.context.opaque_struct_type(&mangled_name);
            struct_type.set_body(&field_types, false);
            self.ctx
                .struct_types
                .insert(mangled_name.clone(), struct_type);

            // Sprint 15: 同时注册 Result<void, E> 类型（用于 Err 构造）
            if !matches!(**ok_type, Type::Void) {
                let void_result_ty = Type::Result {
                    ok_type: Box::new(Type::Void),
                    err_type: err_type.clone(),
                };
                let void_mangled = lency_monomorph::mangling::mangle_type(&void_result_ty);

                if !self.ctx.struct_types.contains_key(&void_mangled) {
                    let void_field_types = vec![
                        self.ctx.context.bool_type().into(),
                        err_type.to_llvm_type(&*self.ctx)?,
                    ];

                    let void_struct = self.ctx.context.opaque_struct_type(&void_mangled);
                    void_struct.set_body(&void_field_types, false);
                    self.ctx
                        .struct_types
                        .insert(void_mangled.clone(), void_struct);
                }
            }
        }

        Ok(())
    }

    /// 递归检查并注册类型中的 Result 类型
    pub(crate) fn register_result_type_if_needed(&mut self, ty: &Type) -> CodegenResult<()> {
        match ty {
            Type::Result { .. } => self.register_result_type(ty),
            Type::Nullable(inner) => self.register_result_type_if_needed(inner),
            Type::Array { element_type, .. } => self.register_result_type_if_needed(element_type),
            Type::Vec(inner) => self.register_result_type_if_needed(inner),
            _ => Ok(()),
        }
    }

    /// 第零遍：注册类型 (opaque) - 跳过泛型定义
    pub(crate) fn register_opaque_types(&mut self, program: &Program) -> CodegenResult<()> {
        for decl in &program.decls {
            if let Decl::Struct {
                name,
                fields,
                generic_params,
                ..
            } = decl
            {
                if !generic_params.is_empty() {
                    continue;
                }

                let struct_type = self.ctx.context.opaque_struct_type(name);
                self.ctx.struct_types.insert(name.clone(), struct_type);

                // 记录字段顺序和类型
                let field_names = fields.iter().map(|f| f.name.clone()).collect();
                self.ctx.struct_fields.insert(name.clone(), field_names);

                let field_types = fields.iter().map(|f| f.ty.clone()).collect();
                self.ctx
                    .struct_field_types
                    .insert(name.clone(), field_types);
            } else if let Decl::Enum {
                name,
                variants,
                generic_params,
                ..
            } = decl
            {
                if !generic_params.is_empty() {
                    continue;
                }

                // 注册 Enum 类型 (opaque)
                let enum_type = self.ctx.context.opaque_struct_type(name);
                self.ctx.struct_types.insert(name.clone(), enum_type);
                self.ctx.enum_types.insert(name.clone());

                // 记录变体信息
                let mut variants_info = Vec::new();
                for variant in variants {
                    match variant {
                        EnumVariant::Unit(v_name) => variants_info.push((v_name.clone(), vec![])),
                        EnumVariant::Tuple(v_name, types) => {
                            variants_info.push((v_name.clone(), types.clone()))
                        }
                    }
                }
                self.ctx.enum_variants.insert(name.clone(), variants_info);
            }
        }
        Ok(())
    }

    /// 第0.5遍：定义 Struct Body
    pub(crate) fn define_struct_bodies(&mut self, program: &Program) -> CodegenResult<()> {
        for decl in &program.decls {
            if let Decl::Struct {
                name,
                fields,
                generic_params,
                ..
            } = decl
            {
                if !generic_params.is_empty() {
                    continue;
                }

                // 上一步已经注册了，直接获取
                let struct_type = self.ctx.struct_types.get(name).unwrap();

                let mut field_types = Vec::new();
                for field in fields {
                    // 这里传递 &*self.ctx 是因为 to_llvm_type 需要 &CodegenContext
                    // 而 self.ctx 是 &mut CodegenContext
                    field_types.push(field.ty.to_llvm_type(&*self.ctx)?);
                }

                struct_type.set_body(&field_types, false);
            }
        }
        Ok(())
    }

    /// 第0.6遍：定义 Enum Body (必须在 Struct Body 之后，以便计算大小)
    pub(crate) fn define_enum_bodies(&mut self, program: &Program) -> CodegenResult<()> {
        for decl in &program.decls {
            if let Decl::Enum {
                name,
                variants,
                generic_params,
                ..
            } = decl
            {
                if !generic_params.is_empty() {
                    continue;
                }

                let enum_type = *self.ctx.struct_types.get(name).unwrap();
                let data_layout = self.ctx.module.get_data_layout();
                let mut max_payload_size = 0;

                for variant in variants {
                    match variant {
                        EnumVariant::Unit(_) => {}
                        EnumVariant::Tuple(_, types) => {
                            // 计算 tuple struct 大小 (考虑对齐)
                            let mut field_types = Vec::new();
                            for ty in types {
                                field_types.push(ty.to_llvm_type(&*self.ctx)?);
                            }
                            // 创建临时 struct type 来获取正确的大小和 layout padding
                            let temp_struct = self.ctx.context.struct_type(&field_types, false);

                            // Inkwell 0.4.0: module.get_data_layout() returns Ref<DataLayout>
                            let target_data =
                                TargetData::create(&data_layout.as_str().to_string_lossy());
                            let size =
                                target_data.get_store_size(&temp_struct.as_basic_type_enum());
                            if size > max_payload_size {
                                max_payload_size = size;
                            }
                        }
                    }
                }

                let tag_type = self.ctx.context.i64_type(); // Tag (i64 for alignment)
                let payload_array_type = self
                    .ctx
                    .context
                    .i8_type()
                    .array_type(max_payload_size as u32);

                // Layout: { tag: i64, payload: [max_size x i8] }
                // Ensures 8-byte alignment for payload
                enum_type.set_body(&[tag_type.into(), payload_array_type.into()], false);

                // Generate Constructors (Values)
                // fn Enum_Variant(fields...) -> Enum
                for (tag_idx, variant) in variants.iter().enumerate() {
                    let (variant_name, field_types_ast) = match variant {
                        EnumVariant::Unit(n) => (n, vec![]),
                        EnumVariant::Tuple(n, t) => (n, t.clone()),
                    };

                    let ctor_name = format!("{}_{}", name, variant_name);

                    // Convert field types to LLVM
                    let mut llvm_param_types = Vec::new();
                    for ty in &field_types_ast {
                        let llvm_ty = ty.to_llvm_type(&*self.ctx)?;
                        llvm_param_types.push(llvm_ty.into()); // BasicTypeEnum -> BasicMetadataTypeEnum
                    }

                    // Constructor Function Type - 返回指针类型
                    let ret_ptr_type = enum_type.ptr_type(inkwell::AddressSpace::default());
                    let fn_type = ret_ptr_type.fn_type(&llvm_param_types, false);

                    let function = self.ctx.module.add_function(&ctor_name, fn_type, None);

                    // Generate Body
                    let basic_block = self.ctx.context.append_basic_block(function, "entry");
                    self.ctx.builder.position_at_end(basic_block);

                    // 1. Malloc Enum (Heap) 而不是 Alloca，以便返回指针
                    let size = enum_type.size_of().unwrap();
                    let malloc = self.ctx.module.get_function("malloc").unwrap();
                    let malloc_call = self
                        .ctx
                        .builder
                        .build_call(malloc, &[size.into()], "malloc_enum")
                        .unwrap();
                    let raw_ptr = malloc_call
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_pointer_value();
                    let enum_ptr = self
                        .ctx
                        .builder
                        .build_bitcast(raw_ptr, ret_ptr_type, "enum_ptr")
                        .unwrap()
                        .into_pointer_value();

                    // 2. Store Tag
                    let tag_ptr = self
                        .ctx
                        .builder
                        .build_struct_gep(enum_type, enum_ptr, 0, "tag_ptr")
                        .unwrap();
                    let tag_val = self.ctx.context.i64_type().const_int(tag_idx as u64, false);
                    self.ctx.builder.build_store(tag_ptr, tag_val).unwrap();

                    // 3. Store Fields (if any)
                    if !field_types_ast.is_empty() {
                        // Get Payload Array Ptr
                        let payload_arr_ptr = self
                            .ctx
                            .builder
                            .build_struct_gep(enum_type, enum_ptr, 1, "payload_arr_ptr")
                            .unwrap();

                        // Create Variant Struct Type for casting
                        let mut variant_llvm_types = Vec::new();
                        for ty in &field_types_ast {
                            variant_llvm_types.push(ty.to_llvm_type(&*self.ctx)?);
                        }
                        let variant_struct_type =
                            self.ctx.context.struct_type(&variant_llvm_types, false);

                        // Bitcast i8* (payload_arr) to { fields... }*
                        let payload_ptr = self
                            .ctx
                            .builder
                            .build_bitcast(
                                payload_arr_ptr,
                                variant_struct_type.ptr_type(inkwell::AddressSpace::default()),
                                "payload_ptr",
                            )
                            .unwrap();

                        // Store params
                        for (i, _arg_ty) in field_types_ast.iter().enumerate() {
                            let param_val = function.get_nth_param(i as u32).unwrap();
                            let field_gep = self
                                .ctx
                                .builder
                                .build_struct_gep(
                                    variant_struct_type,
                                    payload_ptr.into_pointer_value(),
                                    i as u32,
                                    "field_ptr",
                                )
                                .unwrap();
                            self.ctx.builder.build_store(field_gep, param_val).unwrap();
                        }
                    }

                    // 4. Return Pointer (不再 load)
                    self.ctx.builder.build_return(Some(&enum_ptr)).unwrap();
                }
            }
        }
        Ok(())
    }
}
