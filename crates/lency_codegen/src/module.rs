//! Module Code Generation
//!
//! 模块代码生成器，负责生成整个程序

use lency_syntax::ast::{Decl, EnumVariant, Program};

use crate::context::CodegenContext;
use crate::error::CodegenResult;
use crate::function::FunctionGenerator;

/// 模块代码生成器
pub struct ModuleGenerator<'ctx, 'a> {
    ctx: &'a mut CodegenContext<'ctx>,
}

impl<'ctx, 'a> ModuleGenerator<'ctx, 'a> {
    /// 创建模块生成器
    pub fn new(ctx: &'a mut CodegenContext<'ctx>) -> Self {
        Self { ctx }
    }

    /// 生成整个程序
    pub fn generate(&mut self, program: &Program) -> CodegenResult<()> {
        use crate::types::ToLLVMType;

        // 注入运行时函数 (__lency_panic, printf, exit)
        let panic_func =
            crate::runtime::inject_runtime_functions(self.ctx.context, &self.ctx.module);
        self.ctx.panic_func = Some(panic_func);

        // 预定义 malloc: declare i8* @malloc(i64)
        let malloc_type = self
            .ctx
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default())
            .fn_type(&[self.ctx.context.i64_type().into()], false);
        self.ctx.module.add_function("malloc", malloc_type, None);

        // 第零遍：注册类型 (opaque) - 跳过泛型定义
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

        // 第0.5遍：定义 Struct Body
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

        // 第0.6遍：定义 Enum Body (必须在 Struct Body 之后，以便计算大小)
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
                            // 注意: get_store_size 需要 TargetData
                            use inkwell::targets::TargetData;
                            use inkwell::types::BasicType;

                            // Inkwell 0.4.0: module.get_data_layout() returns Ref<DataLayout>
                            // DataLayout doesn't have get_store_size, but TargetData does.
                            // We create TargetData from the string representation.
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

                    // Constructor Function Type: (params) -> Enum (Value)
                    // Note: Returning Struct Value usually uses sret, handled by inkwell?
                    // inkwell fn_type returns FunctionType.
                    // StructType::fn_type handles return by value in signature.
                    let fn_type = enum_type.fn_type(&llvm_param_types, false);

                    let function = self.ctx.module.add_function(&ctor_name, fn_type, None);

                    // Generate Body
                    let basic_block = self.ctx.context.append_basic_block(function, "entry");
                    self.ctx.builder.position_at_end(basic_block);

                    // 1. Alloca Enum (Local)
                    let alloca = self
                        .ctx
                        .builder
                        .build_alloca(enum_type, "enum_instance")
                        .unwrap();

                    // 2. Store Tag
                    let tag_ptr = self
                        .ctx
                        .builder
                        .build_struct_gep(enum_type, alloca, 0, "tag_ptr")
                        .unwrap();
                    let tag_val = self.ctx.context.i64_type().const_int(tag_idx as u64, false);
                    self.ctx.builder.build_store(tag_ptr, tag_val).unwrap();

                    // 3. Store Fields (if any)
                    if !field_types_ast.is_empty() {
                        // Get Payload Array Ptr
                        let payload_arr_ptr = self
                            .ctx
                            .builder
                            .build_struct_gep(enum_type, alloca, 1, "payload_arr_ptr")
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

                    // 4. Load and Return
                    let ret_val = self
                        .ctx
                        .builder
                        .build_load(enum_type, alloca, "ret_val")
                        .unwrap();
                    self.ctx.builder.build_return(Some(&ret_val)).unwrap();
                }
            }
        }

        // 第一遍：声明所有函数（支持前向引用）
        for decl in &program.decls {
            match decl {
                Decl::Function {
                    name,
                    generic_params,
                    params,
                    return_type,
                    ..
                } => {
                    if !generic_params.is_empty() {
                        continue;
                    }

                    self.ctx
                        .function_signatures
                        .insert(name.clone(), return_type.clone());

                    if name == "main" {
                        let func_gen = FunctionGenerator::new(&*self.ctx);
                        // Declare user main as __lency_main
                        func_gen.declare("__lency_main", params, return_type)?;
                    } else {
                        let func_gen = FunctionGenerator::new(&*self.ctx);
                        func_gen.declare(name, params, return_type)?;
                    }
                }

                Decl::ExternFunction {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    self.ctx
                        .function_signatures
                        .insert(name.clone(), return_type.clone());
                    let func_gen = FunctionGenerator::new(&*self.ctx);
                    func_gen.declare(name, params, return_type)?;
                }
                Decl::Struct { .. } => {
                    // 已处理
                }
                Decl::Impl {
                    type_name, methods, ..
                } => {
                    // 声明所有方法（添加隐式 this 参数）
                    for method in methods {
                        if let Decl::Function {
                            name,
                            params,
                            return_type,
                            ..
                        } = method
                        {
                            // 生成 mangled 名称：StructName_methodName
                            // Use mangling to handle generics/primitives correctly
                            let type_str = lency_monomorph::mangling::mangle_type(type_name);
                            let mangled_name = format!("{}_{}", type_str, name);

                            // 构建带 this 指针的参数列表
                            let this_type = type_name.clone();
                            let this_param = lency_syntax::ast::Param {
                                name: "this".to_string(),
                                ty: this_type,
                            };

                            let mut method_params = vec![this_param];
                            method_params.extend_from_slice(params);

                            let func_gen = FunctionGenerator::new(&*self.ctx);
                            // Pass type_str as context so recursive calls know "struct name"
                            // But func_gen.declare/generate might expect strict "Struct" name logic?
                            // Let's check generate. It takes struct_name_context: Option<&str>.
                            // If we pass "int", it logic of splitting might be weird if name is "int_hash".
                            // But previously we fixed splitting logic.
                            func_gen.declare(&mangled_name, &method_params, return_type)?;

                            // 注册函数签名
                            self.ctx
                                .function_signatures
                                .insert(mangled_name.clone(), return_type.clone());
                        }
                    }
                }
                // Trait 定义：阶段1不生成代码，仅注册
                Decl::Trait { .. } => {}
                // Enum 定义：在 to_llvm_type 时按需生成布局，这里跳过
                Decl::Enum { .. } => {}
                Decl::Import { .. } => {}
                Decl::Var {
                    name, ty, value, ..
                } => {
                    // Declare Global
                    let llvm_ty = if let Some(t) = ty {
                        t.to_llvm_type(&*self.ctx)?
                    } else {
                        // Must have type
                        return Err(crate::error::CodegenError::UnsupportedFeature(format!(
                            "Global {} missing type",
                            name
                        )));
                    };

                    let global = self.ctx.module.add_global(llvm_ty, None, name);
                    self.ctx
                        .global_var_types
                        .insert(name.clone(), ty.clone().unwrap()); // ty is Some here

                    // Set Initializer (Assume StructLiteral or Constant)
                    // If value is StructLiteralExpr, generate constant value.
                    // Basic Expr generation returns Value.
                    // But we need Constant.
                    // If it's StructLiteral, we can try to build constant struct.
                    // Simplification: Generate initializer function `__lency_global_init` for complex inits?
                    // Or just handle StructLiteral specially here.
                    // The user case is `my_io = my_io_Module {}`. Empty struct.
                    // Empty struct constant.

                    // Simplified: Set zero initializer if not handled.
                    // For `my_io_Module {}`, it's empty struct.
                    // llvm_ty is struct type.
                    // const_named_struct ?

                    if let lency_syntax::ast::ExprKind::StructLiteral { type_: _, fields } =
                        &value.kind
                    {
                        if fields.is_empty() {
                            // Initializer for empty struct
                            if llvm_ty.is_struct_type() {
                                let struct_ty = llvm_ty.into_struct_type();
                                // Undef or specific constant?
                                // struct type const_named_struct needs values.
                                // inkwell: context.const_struct(&[], false) -> Value
                                // If named struct? `const_named_struct`
                                // struct_ty.const_named_struct(&[])
                                let const_val = struct_ty.const_named_struct(&[]);
                                global.set_initializer(&const_val);
                            }
                        }
                    }
                    // TODO: Better global init handling
                }
            }
        }

        // 第二遍：生成函数体
        let mut func_gen = FunctionGenerator::new(&*self.ctx);
        for decl in &program.decls {
            match decl {
                Decl::Function {
                    name,
                    generic_params,
                    ..
                } => {
                    if !generic_params.is_empty() {
                        continue;
                    }
                    if name == "main" {
                        func_gen.generate(decl, Some("__lency_main"), None)?;
                    } else {
                        func_gen.generate(decl, None, None)?;
                    }
                }
                Decl::ExternFunction { .. } | Decl::Struct { .. } => {
                    continue;
                }
                Decl::Impl {
                    type_name, methods, ..
                } => {
                    // 生成所有方法的函数体
                    for method in methods {
                        if let Decl::Function { name, .. } = method {
                            let type_str = lency_monomorph::mangling::mangle_type(type_name);
                            let mangled_name = format!("{}_{}", type_str, name);
                            func_gen.generate(method, Some(&mangled_name), Some(&type_str))?;
                        }
                    }
                }
                // Trait 定义：不需要生成代码
                Decl::Trait { .. } => {}
                // Enum 定义：不需要生成代码
                Decl::Enum { .. } => {}
                Decl::Import { .. } => {}
                Decl::Var { .. } => {} // Globals generated in declarations pass
            }
        }

        // Generate main wrapper if user main exists
        if let Some(user_main) = self.ctx.module.get_function("__lency_main") {
            let i32_type = self.ctx.context.i32_type();
            let main_type = i32_type.fn_type(&[], false);
            let main_func = self.ctx.module.add_function("main", main_type, None);

            let entry = self.ctx.context.append_basic_block(main_func, "entry");
            self.ctx.builder.position_at_end(entry);

            let call_inst = self
                .ctx
                .builder
                .build_call(user_main, &[], "call_user_main")
                .unwrap();

            // If user main returns int, use it as exit code
            // user_main is FunctionValue. get_type() -> FunctionType. get_return_type() -> Option<BasicTypeEnum>
            let return_type = user_main.get_type().get_return_type();

            if let Some(ret_ty) = return_type {
                if ret_ty.is_int_type() {
                    // Truncate i64 to i32
                    let ret_val = call_inst
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_int_value();
                    let exit_code = self
                        .ctx
                        .builder
                        .build_int_cast(ret_val, i32_type, "exit_code")
                        .unwrap();
                    self.ctx.builder.build_return(Some(&exit_code)).unwrap();
                } else {
                    let zero = i32_type.const_int(0, false);
                    self.ctx.builder.build_return(Some(&zero)).unwrap();
                }
            } else {
                // Void return
                let zero = i32_type.const_int(0, false);
                self.ctx.builder.build_return(Some(&zero)).unwrap();
            }
        }

        Ok(())
    }
}
