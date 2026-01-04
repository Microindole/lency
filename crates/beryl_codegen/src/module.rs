//! Module Code Generation
//!
//! 模块代码生成器，负责生成整个程序

use beryl_syntax::ast::{Decl, Program, Type};

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

        // 注入运行时函数 (__beryl_panic, printf, exit)
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

        // 第零遍：注册所有 Struct 类型（opaque）
        for decl in &program.decls {
            if let Decl::Struct { name, fields, .. } = decl {
                let struct_type = self.ctx.context.opaque_struct_type(name);
                self.ctx.struct_types.insert(name.clone(), struct_type);

                // 记录字段顺序和类型
                let field_names = fields.iter().map(|f| f.name.clone()).collect();
                self.ctx.struct_fields.insert(name.clone(), field_names);

                let field_types = fields.iter().map(|f| f.ty.clone()).collect();
                self.ctx
                    .struct_field_types
                    .insert(name.clone(), field_types);
            }
        }

        // 第0.5遍：定义 Struct Body
        for decl in &program.decls {
            if let Decl::Struct { name, fields, .. } = decl {
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

        // 第一遍：声明所有函数（支持前向引用）
        for decl in &program.decls {
            match decl {
                Decl::Function {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    self.ctx
                        .function_signatures
                        .insert(name.clone(), return_type.clone());

                    if name == "main" {
                        let func_gen = FunctionGenerator::new(&*self.ctx);
                        // Declare user main as __beryl_main
                        func_gen.declare("__beryl_main", params, return_type)?;
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
                            let mangled_name = format!("{}_{}", type_name, name);

                            // 构建带 this 指针的参数列表
                            let this_type = Type::Struct(type_name.clone());
                            let this_param = beryl_syntax::ast::Param {
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
            }
        }

        // 第二遍：生成函数体
        let mut func_gen = FunctionGenerator::new(&*self.ctx);
        for decl in &program.decls {
            match decl {
                Decl::Function { name, .. } => {
                    if name == "main" {
                        func_gen.generate(decl, Some("__beryl_main"))?;
                    } else {
                        func_gen.generate(decl, None)?;
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
                            let mangled_name = format!("{}_{}", type_name, name);
                            func_gen.generate(method, Some(&mangled_name))?;
                        }
                    }
                }
            }
        }

        // Generate main wrapper if user main exists
        if let Some(user_main) = self.ctx.module.get_function("__beryl_main") {
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
