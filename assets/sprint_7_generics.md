# Sprint 7: Beryl 泛型系统实施计划

## 最新状态

- **开始时间**: 2026-01-04
- **当前阶段**: ✅ 阶段1完成，阶段2规划完成
- **总体进度**: 25% (1/4阶段)

## 概述

本Sprint的目标是为Beryl语言实现完整的泛型系统，支持泛型结构体、泛型函数和泛型方法。实现采用**单态化**策略（类似C++模板和Rust泛型）。

## 泛型语法设计

```beryl
// 泛型结构体
struct Box<T> {
    T value;
}

struct Pair<K, V> {
    K first;
    V second;
}

// 泛型函数
T identity<T>(T x) {
    return x;
}

K first<K, V>(Pair<K, V> p) {
    return p.first;
}

// 泛型impl块
impl<T> Box<T> {
    T get() {
        return this.value;
    }
    
    void set(T val) {
        this.value = val;
    }
}

// 使用示例
int main() {
    var box_int: Box<int>;
    box_int.value = 42;
    
    var box_str: Box<string>;
    box_str.value = "Hello";
    
    var pair: Pair<string, int>;
    pair.first = "age";
    pair.second = 25;
    
    var x = identity<int>(10);
    var s = identity<string>("hello");
    
    return 0;
}
```

---

## 实施阶段

### ✅ 阶段1: 语法 & AST 层（已完成）

**状态**: ✅ 完成  
**完成时间**: 2026-01-04

#### 已完成的工作

1. **AST修改**
   - ✅ 新增 `Type::GenericParam(String)` - 表示泛型参数（如`T`）
   - ✅ 保留 `Type::Generic(String, Vec<Type>)` - 表示泛型实例化（如`Box<int>`）
   - ✅ 为 `Decl::Struct` 添加 `generic_params: Vec<String>`
   - ✅ 为 `Decl::Function` 添加 `generic_params: Vec<String>`
   - ✅ 为 `Decl::Impl` 添加 `generic_params: Vec<String>`

2. **Parser修改**
   - ✅ 实现 `generic_params_parser()` - 解析 `<T, U, V>`
   - ✅ 更新 `type_parser()` - 支持泛型类型实例化
   - ✅ 更新 `struct_decl()` - 解析泛型结构体
   - ✅ 更新 `func()` - 解析泛型函数
   - ✅ 更新 `impl_decl()` - 解析泛型impl块

3. **依赖适配**
   - ✅ 修复 `beryl_sema` 中的编译错误
   - ✅ 修复 `beryl_codegen` 中的编译错误
   - ✅ 在模式匹配中添加 `generic_params` 或使用 `..`

4. **验证**
   - ✅ 创建 `examples/test_parser.rs` 验证功能
   - ✅ 成功解析 `struct Box<T>`
   - ✅ 成功解析 `T identity<T>(T x)`
   - ✅ 成功解析 `impl<T> Box<T>`
   - ✅ 项目成功编译

#### 遇到的问题及解决方案

**问题**: Parser测试时栈溢出

**原因**: 
1. 初始使用 `recursive()` 和 `ty.clone()` 导致左递归
2. cargo test 默认栈大小不足

**解决方案**:
1. 移除 `recursive()` 包装，使用 `choice()` 代替 `.or()` 链
2. 限制泛型参数只能是简单标识符（不支持嵌套如`Box<Box<int>>`）
3. 在 `.cargo/config.toml` 中增加测试栈大小到8MB

#### 修改的文件

- `crates/beryl_syntax/src/ast/types.rs` - 新增 `Type::GenericParam`
- `crates/beryl_syntax/src/ast/stmt.rs` - 为Decl添加 `generic_params`
- `crates/beryl_syntax/src/parser/helpers.rs` - 实现泛型解析
- `crates/beryl_syntax/src/parser/decl.rs` - 集成泛型解析
- `.cargo/config.toml` - 增加测试栈大小
- `crates/beryl_sema/src/resolver/decl.rs` - 适配新字段
- `crates/beryl_sema/src/type_check/decl.rs` - 适配新字段
- `crates/beryl_codegen/src/types.rs` - 添加 `GenericParam` 处理

---

### ⏭️ 阶段2: 语义分析（部分完成）

**状态**: 🔄 进行中  
**更新时间**: 2026-01-06

#### 已完成的工作

1. **符号表扩展** (`symbol.rs`)
   - ✅ 新增 `GenericParamSymbol` 结构体
   - ✅ 为 `StructSymbol` 添加 `generic_params: Vec<GenericParamSymbol>` 字段
   - ✅ 为 `FunctionSymbol` 添加 `generic_params: Vec<GenericParamSymbol>` 字段
   - ✅ 新增 `Symbol::GenericParam` 变体
   - ✅ 新增构造函数 `new_generic()` 和 `is_generic()` 方法

2. **Resolver修改** (`resolver/decl.rs`)
   - ✅ `collect_decl`: 收集泛型参数到 `StructSymbol` 和 `FunctionSymbol`
   - ✅ `resolve_decl`: 在解析函数时将泛型参数注册到作用域
   - ✅ `resolve_decl`: 在解析结构体时创建临时作用域注册泛型参数
   - ✅ `resolve_decl`: 在解析impl块时注册泛型参数到方法作用域
   - ✅ 字段类型验证现在可以识别泛型参数（如 `T`）

3. **导出更新** (`lib.rs`)
   - ✅ 导出 `GenericParamSymbol`

#### 待完成的工作

- [ ] **TypeChecker修改**: 验证泛型实例化（如 `Box<int>`）的正确性
- [ ] **错误处理**: 添加 `GenericArityMismatch` 等错误类型
- [ ] **测试**: 编写专门的泛型语义分析测试

---

#### 计划的修改

**1. 符号表扩展** (`symbol.rs`)
-   [ ] **TypeChecker修改**: 验证泛型实例化（如 `Box<int>`）的正确性
-   [ ] **错误处理**: 添加 `GenericArityMismatch` 等错误类型
-   [ ] **测试**: 编写专门的泛型语义分析测试

---

### Phase 3: Monomorphization (Basic) - ✅ Completed
-   [x] `Monomorphizer` Pass stucture
-   [x] `Collector` implementation
-   [x] `Specializer` implementation (generic struct)
-   [x] `Rewriter` implementation
-   [x] Driver integration
-   [x] Verification: `Box<int>` compilation

### Phase 4: Generic Methods - ✅ Completed
-   [x] `generic_impls` collection
-   [x] `Impl` block specialization (`impl Box__int`)
-   [x] Method specialization (`Box__int_get`)
-   [x] Integration Test: `generic_method.brl` (`Box<T>.get()`)

### Phase 5: Generic Free Functions & Inference - 🚧 Pending
-   [ ] Generic Function Calls (`identity<int>(10)`)
-   [ ] Argument Type Inference (`identity(10)` -> `T=int`)
-   [ ] Turbo-fish syntax support in Parser? (Already supported `ident<args>`)

## 自举 (Self-Hosting) 差距分析
要实现 Beryl 自举，当前语言还需要以下关键特性：
1.  **完整标准库 (StdLib)**: 文件 I/O (`File`), 字符串操作 (`String`), 集合 (`HashMap` for SymbolTable).
2.  **Trait 系统 (Interfaces)**: 编译器大量使用多态 (Visitor Pattern, AST Traits). 目前仅支持 generic impl，缺乏 trait bounds (`T: Display`).
3.  **模块系统 (Modules)**: `use`, `import` 支持多文件编译。
4.  **错误处理 (Error Handling)**: `Result<T, E>` 及 `?` 操作符 (语法糖已部分支持，但需要 StdLib 类型支持).
5.  **FFI**: 调用 LLVM C API (或输出文本 IR).
6.  **模式匹配增强**: 支持 Enum variants data match (编译器核心).

**2. Resolver修改** (`resolver/decl.rs`)
- 解析结构体时，将泛型参数注册到作用域
- 解析函数时，将泛型参数注册到作用域
- 实现 `resolve_type()` 方法验证类型引用的有效性

**3. TypeChecker修改** (`type_check/mod.rs`)

- 验证泛型实例化的正确性（如 `Box<int>`）
- 检查类型参数数量是否匹配
- 确保类型参数是有效的具体类型

**4. 错误处理** (`error.rs`)

新增错误类型：
- `UndefinedGenericParam` - 未定义的泛型参数
- `GenericArityMismatch` - 泛型参数数量不匹配
- `NotAGenericType` - 对非泛型类型使用类型参数
- `GenericParamShadowing` - 泛型参数名称冲突

#### 测试计划

**单元测试**:
- ✅ 泛型结构体符号收集
- ✅ 泛型函数符号收集
- ✅ 类型引用验证
- ✅ 泛型实例化验证
- ✅ 错误情况测试

**集成测试**:
- 创建 `tests/integration/generics/sema_basic.brl`
- 端到端验证语义分析流程

---

### ⏭️ 阶段3: 代码生成（单态化）

**状态**: 📋 待规划  
**预计工期**: 5-7天

#### 目标

实现泛型的单态化代码生成：

1. **结构体单态化**
   - `Box<T>` + `T=int` → 生成 `struct Box_int`
   - `Box<T>` + `T=string` → 生成 `struct Box_string`

2. **函数单态化**
   - `identity<T>` + `T=int` → 生成 `identity_int()`
   - `identity<T>` + `T=string` → 生成 `identity_string()`

3. **方法单态化**
   - `impl<T> Box<T>` 中的方法对每个具体类型生成独立版本

#### 实现策略

**单态化收集器**:
- 遍历AST，收集所有泛型类型的实例化（如 `Box<int>`, `Box<string>`）
- 为每个实例化生成唯一的具体类型名称

**代码生成**:
- 对每个泛型结构体的实例化，生成独立的LLVM结构体
- 对每个泛型函数的实例化，生成独立的LLVM函数
- 替换所有泛型类型引用为具体类型

**名称改写**:
- `Box<int>` → `Box_int`
- `Box<string>` → `Box_string`
- `identity<int>` → `identity_int`

---

### ⏭️ 阶段4: 集成测试与验证

**状态**: 📋 待规划  
**预计工期**: 2-3天

#### 测试用例

**基础测试**:
```beryl
struct Box<T> {
    T value;
}

int main() {
    var b: Box<int>;
    b.value = 42;
    return b.value;
}
```

**复杂测试**:
```beryl
struct Pair<K, V> {
    K first;
    V second;
}

T identity<T>(T x) {
    return x;
}

impl<T> Pair<T, T> {
    T sum() {
        return this.first + this.second;
    }
}

int main() {
    var p: Pair<int, int>;
    p.first = 10;
    p.second = 20;
    
    var result = identity<int>(p.sum());
    return result;  // 应返回30
}
```

**错误测试**:
- 类型参数数量不匹配
- 使用未定义的泛型参数
- 对非泛型类型使用类型参数

---

## 技术细节

### 单态化策略

借鉴Rust和C++的做法：

1. **编译时展开**: 每个泛型实例化生成独立的代码
2. **优点**: 运行时性能最优，无运行时开销
3. **缺点**: 代码膨胀（每个实例化都有独立副本）

### 类型系统集成

泛型参数的作用域：
- **结构体泛型参数**: 在字段定义和关联impl块中有效
- **函数泛型参数**: 在参数列表、返回类型和函数体中有效
- **impl块泛型参数**: 在方法定义中有效

### 限制和未来扩展

**当前限制**:
- ❌ 不支持嵌套泛型实例化（如 `Box<Box<int>>`）
- ❌ 不支持泛型约束（trait bounds）
- ❌ 不支持默认类型参数
- ❌ 不支持可变参数泛型

**未来可扩展**:
- ✨ Trait系统 + 泛型约束
- ✨ 嵌套泛型支持
- ✨ 高阶类型（Higher-Kinded Types）
- ✨ 泛型类型推导优化

---

## 参考资料

### 相关文件
- 详细实现计划: `.gemini/antigravity/brain/.../implementation_plan.md`
- 任务清单: `.gemini/antigravity/brain/.../task.md`
- 完成报告: `.gemini/antigravity/brain/.../walkthrough.md`

### 设计参考
- Rust泛型系统
- C++模板系统
- Go语言泛型（Go 1.18+）

---

## 总结

**已完成**:
- ✅ 阶段1: 语法和AST层完全实现
- ✅ Parser成功解析泛型语法
- ✅ 项目编译成功
- ✅ 功能验证通过

**进行中**:
- 📋 阶段2: 语义分析规划完成

**待完成**:
- ⏭️ 阶段2: 语义分析实现
- ⏭️ 阶段3: 代码生成（单态化）
- ⏭️ 阶段4: 集成测试

**预计完成时间**: 2周内完成所有4个阶段
