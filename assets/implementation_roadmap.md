# Beryl 语言实现路线图 v2.4

> **当前状态**: v0.9 (错误处理系统完成)
> **设计哲学**: 简洁 (Concise) · 规范 (Standard) · 清晰 (Clear)

---

## 🏛️ 已完成里程碑 (Completed)

### 阶段 1-4: 语言基石
- ✅ **基础架构**: Lexer, Parser, AST, Codegen (LLVM)。
- ✅ **核心类型**: `int`, `float`, `bool`, `string`。
- ✅ **控制流**: `if`, `while`, `for` (Classic & For-in), `match` (Int)。
- ✅ **集合**: 数组 (`[T; N]`) 与 动态数组 (`vec![...]`)。

### 阶段 5: 结构体与方法 (Structs & Methods)
- ✅ **结构体**: C 风格结构体定义与初始化。
- ✅ **方法**: `impl` 块，支持方法定义。
- ✅ **隐式 This**: 方法内字段的简写访问。

### 阶段 6: 空安全系统 (Null Safety)
- ✅ **显式空类型**: `T?` 表示可空，`T` 默认非空。
- ✅ **安全操作符**: `?.` (安全调用) 与 `??` (Elvis)。
- ✅ **流敏感分析**: `if x != null` 智能转换 (Smart Casts)。

### 阶段 7: 泛型 (Generics)
- ✅ **泛型结构体**: `struct Box<T> { ... }`。
- ✅ **泛型函数**: `fn identity<T>(x: T) -> T`。
- ✅ **单态化实现**: 编译时为具体类型生成代码，零运行时开销。

### 阶段 8: 特性 (Traits)
- ✅ **Trait 定义**: 接口定义与方法签名。
- ✅ **Trait 实现**: `impl Trait for Type`。
- ✅ **泛型约束**: `fn foo<T: Trait>(x: T)`。

### 阶段 9: 错误处理 (Error Handling)
- ✅ **Result 类型**: `int!` 语法糖 (等价于 `Result<int, Error>`)。
- ✅ **构造器**: `Ok(val)` 与 `Err(err)`。
- ✅ **Try 操作符**: `expr?` 自动解包或提前返回。

---

## 🛣️ 下一步计划 (Next Steps)

### **阶段 10: 代数数据类型 (ADTs) 与 模式匹配** 🚧 *Next Sprint*
*目标: 强大的状态建模工具。*

#### 10.1 通用枚举 (Enums)
- **定义**: `enum Option<T> { Some(T), None }`。
- **内存布局**: Tagged Union 实现。

#### 10.2 模式匹配升级
- **解构匹配**: `match` 支持 Struct 和 Enum 解构。
- **穷尽性检查**: 确保所有分支都被处理。

---

### **阶段 11: 闭包与函数式编程**
*目标: 提升语言表达力。*

- **函数类型**: `fn(int) -> bool`。
- **闭包 (Closures)**: 捕获环境变量的匿名函数。

---

### **阶段 12: 标准库与 I/O**
*目标: 构建实用的应用程序。*

- **文件系统**: `File` API。
- **字符串处理**: `Split`, `Join` 等。

---

## 📅 开发时间表 (更新)

| 冲刺 (Sprint) | 核心任务 | 状态 |
|--------|-------|-----------|
| **Sprint 1-6** | 基础/OOP/空安全 | ✅ 完成 |
| **Sprint 7** | 泛型 (Generics) | ✅ 完成 |
| **Sprint 8** | Trait 系统 | ✅ 完成 |
| **Sprint 9** | 错误处理 | ✅ 完成 |
| **Sprint 10** | **ADTs & 模式匹配** | 🏁 即将开始 |
