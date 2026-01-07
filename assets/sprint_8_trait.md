# Sprint 8: Traits (接口与约束) - 极简实施计划

## 1. 核心目标 (Objectives)

本 Sprint 的核心是引入 **Traits (特质/接口)** 系统，这是 Beryl 类型系统的最后一块基石。
在此过程中，我们严格遵循**最少新增关键字原则**。

> **关键字策略**: 仅新增 **1个** 关键字: `trait`.

---

## 2. 语法规范 (Syntax Specification)

### 2.1 Trait 定义 (新增 `trait` 关键字)
使用 `trait` 关键字定义一组方法签名。

```beryl
trait Comparable<T> {
    bool equals(T other);
}

trait ToString {
    string to_string();
}
```

### 2.2 Impl 实现 (复用 `impl` 关键字)
复用现有的 `impl` 关键字，扩展其语法以支持 Trait 实现。

```beryl
struct Point {
    int x;
    int y;
}

// 语法扩展: impl trait_name for type_name
impl ToString for Point {
    string to_string() {
        // 复用现有的 this 关键字访问成员
        return "Point";
    }
}
```

### 2.3 泛型约束 (Trait Bounds)
在泛型参数后使用 `:` 指定约束。

```beryl
// T 必须实现 Comparable<T>
bool not_equal<T: Comparable<T>>(T a, T b) {
    return !a.equals(b);
}
```

---

## 3. 详细实施步骤 (Implementation Roadmap)

### 🔹 阶段 1: 语法解析与 AST (Syntax & AST)
**预计工期**: 1-2 天
**目标**: 让 Parser 能够理解 `trait` 和 `impl Trait for Type`。

1.  **Lexer 修改** (`lexer.rs`)
    -   新增 `Token::Trait` ("trait").

2.  **AST 扩展** (`ast/decl.rs`)
    -   新增 `Decl::Trait { name, methods, generic_params }`.
    -   更新 `Decl::Impl`: 将 `trait_ref: Option<Type>` 加入结构。

3.  **Parser 修改** (`parser/decl.rs`)
    -   实现 `trait_decl`.
    -   更新 `impl_decl` 解析 `for Type` 部分。
    -   更新 `generic_params` 解析 `<T: Bound>`.

### 🔹 阶段 2: 语义分析基础 (Semantic Analysis - Symbols)
**预计工期**: 2 天
**目标**: 注册 Trait 符号。

1.  **Symbol Table**: 新增 `TraitSymbol`.
2.  **Resolution**: 解析 `impl` 块，确保 Trait 存在且方法签名匹配。

### 🔹 阶段 3: 泛型约束检查 (Constraint Checking)
**预计工期**: 3 天
**目标**: 确保泛型调用安全。

1.  **Type Checking**: 在调用泛型函数时，检查实参是否满足 Trait 约束。
2.  **Method Call**: 允许在泛型参数 `T` 上调用 Trait 定义的方法。

### 🔹 阶段 4: 代码生成与单态化 (Codegen)
**预计工期**: 2 天
**目标**: 静态分发。

1.  **静态分发**: 通过单态化直接调用具体实现的方法，无运行时开销。

---

## 4. 验证计划

我们将编写一个核心测试 `tests/traits/basic.brl`，仅涵盖最基础的定义和实现。

```beryl
trait Greeter {
    void greet();
}

struct User { string name; }

impl Greeter for User {
    void greet() {
        print("Hello");
    }
}

fn run<T: Greeter>(T u) {
    u.greet();
}
```

## 5. 风险控制
- **避免复杂特性**: 第一阶段不支持 Trait 继承 (`trait A: B`)。
- **避免新关键字**: 不引入 `Self`, `super`, `interface` 等。
- **孤儿规则**: 暂不强制限制实现位置。
