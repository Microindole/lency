use super::ModuleGenerator;
use crate::error::CodegenResult;
use crate::function::FunctionGenerator;
use crate::types::ToLLVMType;
use lency_syntax::ast::{Decl, Program};

impl<'ctx, 'a> ModuleGenerator<'ctx, 'a> {
    /// 注入运行时函数 (__lency_panic, printf, exit, malloc)
    pub(crate) fn inject_runtime(&mut self) -> CodegenResult<()> {
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

        Ok(())
    }

    /// 第一遍：声明所有函数（支持前向引用）
    pub(crate) fn declare_functions(&mut self, program: &Program) -> CodegenResult<()> {
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

                    // Sprint 15: 预注册函数返回值中的 Result 类型
                    self.register_result_type_if_needed(return_type)?;

                    // 预注册参数中的 Result 类型
                    for param in params {
                        self.register_result_type_if_needed(&param.ty)?;
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
                    // Sprint 15: For Result<T,E> impl, register struct type first
                    if let lency_syntax::ast::Type::Generic(name, args) = type_name {
                        if name == "Result" && args.len() == 2 {
                            let result_ty = lency_syntax::ast::Type::Result {
                                ok_type: Box::new(args[0].clone()),
                                err_type: Box::new(args[1].clone()),
                            };
                            self.register_result_type(&result_ty)?;
                        }
                    }

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
                            let type_str = lency_monomorph::mangling::mangle_type(type_name);
                            let mangled_name = format!("{}_{}", type_str, name);

                            // 构建带 this 指针的参数列表
                            let this_type = match type_name {
                                lency_syntax::ast::Type::Generic(name, args)
                                    if name == "Result" && args.len() == 2 =>
                                {
                                    lency_syntax::ast::Type::Result {
                                        ok_type: Box::new(args[0].clone()),
                                        err_type: Box::new(args[1].clone()),
                                    }
                                }
                                _ => type_name.clone(),
                            };
                            let this_param = lency_syntax::ast::Param {
                                name: "this".to_string(),
                                ty: this_type,
                            };

                            let mut method_params = vec![this_param];
                            method_params.extend_from_slice(params);

                            let func_gen = FunctionGenerator::new(&*self.ctx);
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

                    // Set Initializer (Assume StructLiteral or Constant for now)
                    if let lency_syntax::ast::ExprKind::StructLiteral { fields, .. } = &value.kind {
                        if fields.is_empty() {
                            // Initializer for empty struct
                            if llvm_ty.is_struct_type() {
                                let struct_ty = llvm_ty.into_struct_type();
                                let const_val = struct_ty.const_named_struct(&[]);
                                global.set_initializer(&const_val);
                            }
                        }
                    }
                    // TODO: Better global init handling
                }
            }
        }
        Ok(())
    }

    /// 第二遍：生成函数体
    pub(crate) fn generate_function_bodies(&mut self, program: &Program) -> CodegenResult<()> {
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
                Decl::Trait { .. } => {}
                Decl::Enum { .. } => {}
                Decl::Import { .. } => {}
                Decl::Var { .. } => {} // Globals generated in declarations pass
            }
        }
        Ok(())
    }

    /// Generate main wrapper if user main exists
    pub(crate) fn generate_main_wrapper(&mut self) -> CodegenResult<()> {
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
