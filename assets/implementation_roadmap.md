# Beryl 语言实现路线图 v2.2

> **当前状态**: v0.4 (控制流与基础架构完成)
> **设计哲学**: 简洁 (Concise) · 规范 (Standard) · 清晰 (Clear)

---

## 🏛️ v0.x 基石 (已完成)

语言的核心基础设施已搭建完毕，具备了图灵完备性。

### 核心特性 (Core Features)
- ✅ **编译器管道**: 完整的 Lexer → Parser → Sema → Codegen (LLVM) 流程。
- ✅ **类型系统**:
    - 基础类型: `int`, `float`, `bool`, `string`。
    - **自动类型提升**: `int` + `float` -> `float`，减少繁琐的转换代码。
- ✅ **控制流**:
    - `if`/`else` 语句。
    - `while` 循环。
    - `for` 循环 (C 风格: `for var i=0; i<10; i=i+1`)。
    - `break` / `continue` 跳转控制。
    - `return` 返回值。
- ✅ **模式匹配**:
    - `match` 表达式 (Phase 1: 整数匹配)，编译为高效的 LLVM `switch` 指令。
- ✅ **函数与变量**:
    - `fn` 函数定义，支持递归。
    - `var` 变量声明，支持类型推导。
    - 完整的作用域管理 (块级作用域)。
- ✅ **内建功能**:
    - `print(x)`: 多态打印函数，自动处理所有基础类型。

---

## 🛣️ 通往 v1.0 之路

### **阶段 3: 集合类型 (数组)** 🚧 *Next Sprint*
*目标: 简单、安全、高效的数据聚合。*

#### 3.1 固定大小数组 (Fixed-size Arrays)
- **语法**:
  ```beryl
  var arr: [int; 5] = [1, 2, 3, 4, 5]
  print(arr[0])
  ```
- **特性**: 栈上分配 (Value Semantics)，编译期长度检查，运行时边界检查 (Bounds Check)。
- **实现**: 映射为 LLVM Array Type `[N x T]`。

#### 3.2 动态数组 (Vectors)
- **语法**:
  ```beryl
  var v = vec![1, 2, 3]
  v.push(4)
  print(v.len())
  ```
- **特性**: 堆上分配，自动扩容。需引入基础运行时 (Runtime) 支持 `malloc`/`realloc`。

---

### **阶段 4: 结构体与方法**
*目标:清晰的数据建模与封装。*

#### 4.1 C 风格结构体
- **语法**:
  ```beryl
  struct Point {
      int x
      int y
  }
  var p = Point { x: 10, y: 20 }
  ```
- **特性**: 内存布局兼容 C ABI，零开销抽象。

#### 4.2 方法 (Methods)
- **语法**:类似于 Rust/Go 的实现块 (Implementation Block)。
  ```beryl
  impl Point {
      fn distance(self, other: Point) -> float { ... }
  }
  p.distance(p2)
  ```
- **实现**: 静态分发 (Static Dispatch)，本质上是语法糖 `Point_distance(self, other)`。

---

### **阶段 5: 空安全系统 (Null Safety)** 🌟 *核心卖点*
*目标: 彻底消除空指针异常 (The Billion Dollar Mistake)。*

#### 5.1 可空类型
- **概念**: 默认情况下类型不可为 null。只有标记为 `?` 的类型才能持有 null。
- **语法**: `string?`, `int?`。

#### 5.2 流敏感分析 (Flow Analysis)
- **智能转换 (Smart Casts)**:
  ```beryl
  fn process(s: string?) {
      if s != null {
          print(s.length()) // 编译器知道此处 s 是 string (非空)
      }
  }
  ```
- **安全操作符**: `?.` (安全调用), `??` (Elvis 操作符)。

---

### **阶段 6: 模块化与可见性**
*目标: 支持大型项目开发，遵循开闭原则。*

- **文件即模块**: 每个 `.brl` 文件是一个模块。
- **可见性**: 默认私有。使用 `pub` 关键字导出及其符号。
- **导入**: `import std.io` 或 `from std.math import min`。

---

### **阶段 7: 泛型 (Generics)**
*目标: 代码复用，无需运行时开销。*

- **语法**: `struct Box<T> { T value }`，`fn map<T, U>(arr: [T], f: fn(T)->U) -> [U]`。
- **实现**: 单态化 (Monomorphization)。编译期为每个具体类型生成代码，运行效率等同于手写。

---

## 🔮 未来展望 (Post-v1.0)

- **内存管理**: 引入 RAII (资源获取即初始化) 或轻量级 GC。目前依靠严格的作用域和值语义管理内存。
- **工具链**:
    - `bfmt`: 官方代码格式化工具 (Opinionated Formatter)。
    - `bpm`: 包管理器。
- **FFI**: 完善的 C 语言互操作接口。

---

## 📅 开发时间表 (预估)

| 冲刺 (Sprint) | 核心任务 | 预估工时 |
|--------|-------|-----------|
| **Sprint 3 (进行中)** | **数组 (固定 & 动态)** | 1 周 |
| **Sprint 4** | 结构体 (Structs) | 2 周 |
| **Sprint 5** | 方法与 UFCS | 1 周 |
| **Sprint 6** | **空安全系统 (Null Safety)** | 3 周 |
| **Sprint 7** | 模块系统 | 1.5 周 |
| **Sprint 8** | 基础泛型 | 3 周 |

## 🏗️ 技术债务与基础设施

- [ ] **内存泄漏**: 目前字符串操作存在内存泄漏，需要引入 `Drop` 机制或简单的 GC。
- [ ] **错误报告**: 集成 `ariadne` 库，提供美观、带上下文的报错信息。
- [ ] **性能优化**: 引入 LLVM 优化 Pass (`-O2`, `-O3`)。
