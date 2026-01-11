pub mod collector;
pub mod mangling;
pub mod rewriter;
pub mod specializer;

use self::collector::Collector;
use self::mangling::mangle_type;
use self::rewriter::Rewriter;
use self::specializer::Specializer;
use beryl_syntax::ast::{Decl, Program, Type};
use std::collections::{HashMap, HashSet};

/// 单态化 Pass
/// 将泛型 AST 转换为具体化的 AST
pub struct Monomorphizer {
    /// 记录已经生成的具体类型名称，避免重复生成
    generated_types: HashSet<String>,
    /// 收集到的新声明
    new_decls: Vec<Decl>,
    /// 原始泛型定义缓存 (name -> Decl)
    generic_definitions: HashMap<String, Decl>,
    /// 泛型 Impl 定义缓存 (struct_name -> List[Impl Decl])
    generic_impls: HashMap<String, Vec<Decl>>,
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            generated_types: HashSet::new(),
            new_decls: Vec::new(),
            generic_definitions: HashMap::new(),
            generic_impls: HashMap::new(),
        }
    }

    pub fn process(&mut self, program: Program) -> Program {
        // 1. 分离泛型定义（Templates）和具体定义（Concrete）
        let mut concrete_decls = Vec::new();

        for decl in program.decls {
            match &decl {
                Decl::Struct {
                    name,
                    generic_params,
                    ..
                } if !generic_params.is_empty() => {
                    self.generic_definitions.insert(name.clone(), decl);
                }
                Decl::Function {
                    name,
                    generic_params,
                    ..
                } if !generic_params.is_empty() => {
                    self.generic_definitions.insert(name.clone(), decl);
                }
                Decl::Enum {
                    name,
                    generic_params,
                    ..
                } if !generic_params.is_empty() => {
                    self.generic_definitions.insert(name.clone(), decl);
                }
                // Impl blocks with generics?
                // Currently Beryl parser might have Impl<T> Box<T>.
                // We treat Impl as generic if it has params.
                Decl::Impl {
                    type_name,
                    generic_params,
                    ..
                } => {
                    // Collect all impl blocks for potentially generic structs.
                    // If generic_params is not empty, it's definitely a generic impl template.
                    // Even if generic_params is empty, it might be `impl Box<int>`, which we treat as concrete for now,
                    // UNLESS we want to attach it to the Monomorphization process?
                    // "Concrete" decls are simply kept.
                    // "Generic" templates are stored and instantiated on demand.
                    // `impl<T> Box<T>` is a template.
                    // `impl Box<int>` is concrete.

                    if !generic_params.is_empty() {
                        self.generic_impls
                            .entry(type_name.clone())
                            .or_default()
                            .push(decl.clone());
                    } else {
                        concrete_decls.push(decl);
                    }
                }
                _ => concrete_decls.push(decl),
            }
        }

        // 2. 收集初始实例化需求 (from concrete code)
        let mut collector = Collector::new();
        for decl in &concrete_decls {
            collector.collect_decl(decl);
        }

        // 3. Worklist Algorithm: 持续生成，直到没有新的实例化为止
        let mut type_worklist: Vec<Type> = collector.instantiations.into_iter().collect();
        let mut func_worklist: Vec<(String, Vec<Type>)> =
            collector.function_instantiations.into_iter().collect();

        while !type_worklist.is_empty() || !func_worklist.is_empty() {
            // Process Types
            while let Some(ty) = type_worklist.pop() {
                let mangled_name = mangle_type(&ty);

                // 如果已经处理过（生成的集合中已有），则跳过
                if self.generated_types.contains(&mangled_name) {
                    continue;
                }
                self.generated_types.insert(mangled_name.clone());

                // 尝试特化
                if let Type::Generic(name, args) = &ty {
                    // 处理结构体特化
                    if let Some(template) = self.generic_definitions.get(name) {
                        let new_decl = self.specialize_template(template, args, &mangled_name);

                        // 从新生成的代码中收集新的实例化需求
                        let mut sub_collector = Collector::new();
                        sub_collector.collect_decl(&new_decl);
                        for new_ty in sub_collector.instantiations {
                            let new_mangled = mangle_type(&new_ty);
                            if !self.generated_types.contains(&new_mangled) {
                                type_worklist.push(new_ty);
                            }
                        }
                        for new_func in sub_collector.function_instantiations {
                            // Function mangling logic (same as type for now)
                            let dummy_ty = Type::Generic(new_func.0.clone(), new_func.1.clone());
                            let new_mangled = mangle_type(&dummy_ty);
                            if !self.generated_types.contains(&new_mangled) {
                                func_worklist.push(new_func);
                            }
                        }

                        self.new_decls.push(new_decl);

                        // Also generate corresponding Impl blocks
                        if let Some(impls) = self.generic_impls.get(name) {
                            for impl_decl in impls {
                                let new_impl =
                                    self.specialize_template(impl_decl, args, &mangled_name);

                                let mut sub_collector = Collector::new();
                                sub_collector.collect_decl(&new_impl);
                                for new_ty in sub_collector.instantiations {
                                    let new_mangled = mangle_type(&new_ty);
                                    if !self.generated_types.contains(&new_mangled) {
                                        type_worklist.push(new_ty);
                                    }
                                }
                                for new_func in sub_collector.function_instantiations {
                                    let dummy_ty =
                                        Type::Generic(new_func.0.clone(), new_func.1.clone());
                                    let new_mangled = mangle_type(&dummy_ty);
                                    if !self.generated_types.contains(&new_mangled) {
                                        func_worklist.push(new_func);
                                    }
                                }
                                self.new_decls.push(new_impl);
                            }
                        }
                    }
                }
            }

            // Process Functions
            while let Some((func_name, args)) = func_worklist.pop() {
                // Reuse mangle_type logic for function name
                let dummy_ty = Type::Generic(func_name.clone(), args.clone());
                let mangled_name = mangle_type(&dummy_ty);

                if self.generated_types.contains(&mangled_name) {
                    continue;
                }
                self.generated_types.insert(mangled_name.clone());

                if let Some(template) = self.generic_definitions.get(&func_name) {
                    let new_decl = self.specialize_template(template, &args, &mangled_name);

                    let mut sub_collector = Collector::new();
                    sub_collector.collect_decl(&new_decl);

                    for new_ty in sub_collector.instantiations {
                        let new_mangled = mangle_type(&new_ty);
                        if !self.generated_types.contains(&new_mangled) {
                            type_worklist.push(new_ty);
                        }
                    }
                    for new_func in sub_collector.function_instantiations {
                        let dummy_ty = Type::Generic(new_func.0.clone(), new_func.1.clone());
                        let new_mangled = mangle_type(&dummy_ty);
                        if !self.generated_types.contains(&new_mangled) {
                            func_worklist.push(new_func);
                        }
                    }
                    self.new_decls.push(new_decl);
                }
            }
        }

        // 4. 重写原始代码中的泛型引用 (Box<int> -> Box__int)
        //    同时，新生成的代码 (new_decls) 虽然已经在 specialize 时替换了 T->int，
        //    但其中包含的 `Type::Generic("Vec", [int])` 仍然是 Generic 类型。
        //    我们需要把所有生成的 Decl 也过一遍 Rewriter。

        let rewriter = Rewriter::new();

        let mut final_decls = Vec::new();

        // Rewrite concrete decls
        for decl in concrete_decls {
            final_decls.push(rewriter.rewrite_decl(decl));
        }

        // Rewrite generated decls (because specialization produced Type::Generic<int>, rewriter turns it into Type::Struct(Box__int))
        for decl in self.new_decls.drain(..) {
            final_decls.push(rewriter.rewrite_decl(decl));
        }

        Program { decls: final_decls }
    }

    fn specialize_template(&self, template: &Decl, args: &[Type], mangled_name: &str) -> Decl {
        let generic_params = match template {
            Decl::Struct { generic_params, .. } => generic_params,
            Decl::Function { generic_params, .. } => generic_params,
            Decl::Impl { generic_params, .. } => generic_params,
            Decl::Enum { generic_params, .. } => generic_params,
            _ => return template.clone(), // Should not happen
        };

        // Build Type Map: T -> int, U -> string
        let mut type_map = HashMap::new();
        for (i, param) in generic_params.iter().enumerate() {
            if i < args.len() {
                type_map.insert(param.name.clone(), args[i].clone());
            }
        }

        let specializer = Specializer::new(type_map);
        let mut specialized = specializer.specialize_decl(template);

        // Update name
        match &mut specialized {
            Decl::Struct { name, .. } => *name = mangled_name.to_string(),
            Decl::Function { name, .. } => *name = mangled_name.to_string(),
            Decl::Impl { type_name, .. } => *type_name = mangled_name.to_string(),
            Decl::Enum { name, .. } => *name = mangled_name.to_string(),
            _ => {}
        }

        specialized
    }
}
