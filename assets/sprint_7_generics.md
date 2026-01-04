# Sprint 7: 泛型 (Generics) 实施计划

## 🎯 目标
实现 Beryl 的泛型系统，允许定义参数化类型（如 `struct Box<T>`）和泛型函数（如 `fn id<T>(x: T)`），并通过单态化（Monomorphization）生成高效代码。

## 📋 任务清单

### 1. 语法 (Syntax)
- [ ] **Parser 更新**:
    - 支持解析泛型结构体定义: `struct Name<T, U> { ... }`.
    - 支持解析泛型函数定义: `fn name<T>(...) { ... }`.
    - 支持解析类型实例化: `Name<int>`, `func<string>()`.
- [ ] **AST 更新**:
    - `Decl::Struct` 和 `Decl::Function` 增加 `generic_params: Vec<String>` 字段。
    - `Type` 枚举增加 `Generic(String)` (用于 T) 和 `Instance(String, Vec<Type>)` (用于 Box<int>).

### 2. 语义分析 (Semantics)
- [ ] **符号表 (Symbol Table)**:
    - 记录泛型参数 `T`，将其视为当前作用域内的合法类型。
    - 泛型类型/函数注册时不进行完整检查，而是在实例化时检查。
- [ ] **类型检查**:
    - 验证泛型参数数量匹配。
    - 在实例化点（Instantiation Point）替换具体的类型参数。

### 3. 代码生成 (Codegen) - 单态化
- [ ] **泛型结构体**:
    - 当遇到 `Box<int>` 时，生成名为 `Box_int` 的 LLVM Struct。
    - 缓存已生成的实例，避免重复生成。
- [ ] **泛型函数**:
    - 类似 C++ 模板，为每个不同的具体类型参数组合生成一个新的函数副本。
    - 名称修饰（Mangling）: `id_int`, `id_string`。

### 4. 验证 (Verification)
- [ ] **测试用例**:
    - `generic_struct.brl`: 定义并使用 `Box<T>`。
    - `generic_func.brl`: 定义泛型函数并在不同类型上调用。
    - `vec_generic.brl`: 尝试定义 `Vec<T>` (如果可能的话，或作为后续目标)。

## ⚠️ 风险与挑战
- **类型系统复杂性**: 引入 `T` 后，类型相等性判断变复杂。
- **嵌套泛型**: `Box<Box<int>>` 的解析和生成需要小心递归处理。
- **标准库迁移**: 现有的 `vec` 是硬编码的，迁移到真正的泛型 `Vec<T>` 可能涉及 Runtime 的大幅修改。

## 📅 时间计划
- **Day 1**: AST 与 Parser 修改。
- **Day 2**: 语义分析与类型参数追踪。
- **Day 3-4**: 单态化代码生成逻辑。
- **Day 5**: 测试与修复。
