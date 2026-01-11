# 下一步计划：泛型约束使用与虚函数表 (VTable)

根据 Sprint 8 规划，我们已经完成了 Trait 的**基础定义**、**实现解析**以及**泛型约束的语法声明与存在性检查**。
接下来的核心任务是让泛型约束真正发挥作用：**允许在泛型参数 T 上调用方法**。

## 1. 目标 (Objectives)

使以下代码能够通过语义分析并正确运行：

```beryl
trait Greeter {
    void greet();
}

// 泛型函数接受受约束的 T
void run<T: Greeter>(T u) {
    u.greet(); // 目前编译器会报错 "Method greet not found on type T"
}
```

## 2. 实施步骤 (Implementation Steps)

### 2.1 语义分析升级 (Semantic Analysis - Pass 3)
**文件**: `crates/beryl_sema/src/type_check/expr.rs`

1.  **修改方法调用检查 (`check_method_call`)**:
    *   当接收者 (`receiver`) 的类型是 `Type::GenericParam(T)` 时：
    *   查找符号表中 `T` 的 `GenericParamSymbol`。
    *   检查其 `bound` (例如 `Greeter`)。
    *   在 `bound` 指向的 `TraitSymbol` 中查找方法 `greet`。
    *   如果找到，允许调用，并标记该调用为 "Trait Method Call"。

### 2.2 单态化升级 (Monomorphizer)
**文件**: `crates/beryl_sema/src/monomorphize/specializer.rs`

1.  **静态分发 (Static Dispatch)**:
    *   当前单态化将 `T` 替换为 `User`。
    *   由于 Beryl 采用 Struct Value Semantics，替换后 `u.greet()` 会变成 `User` 实例调用 `greet`。
    *   我们需要确保单态化后的代码能正确解析到 `impl Greeter for User` 中的 `greet` 方法。
    *   **关键点**: 如果 `impl` 块中的方法被改名（mangling），调用处也需要更新。或者，我们让单态化后的 AST 保持 `u.greet()`，依赖后续 Pass 重新解析到具体方法。

2.  **名称重整 (Mangling)**:
    *   Trait 方法在 impl 块中可能会被重整为 `User__greet` 吗？
    *   如果是，单态化时需要将 `u.greet()` 指向 `User__greet`。

### 2.3 虚函数表 (VTable) - *可选/后续*
如果需要支持**动态分发** (`Box<dyn Trait>`)，则需要 VTable。
但根据 Sprint 8 极简原则，优先支持 **静态分发 (Monomorphization)**。
本阶段只需保证 `run<MyInt>(i)` 这种单态化调用工作正常。

## 3. 验证计划

创建测试 `tests/integration/traits/constraint_usage.brl`:

```beryl
trait Display {
    void show();
}

struct MyInt { int val; }

impl Display for MyInt {
    void show() { print(this.val); }
}

void print_it<T: Display>(T item) {
    item.show(); // 关键测试点
}

void main() {
    var i = MyInt { val: 42 };
    print_it<MyInt>(i);
}
```

## 4. 提交信息 (Commit Message)
(将在任务完成后提供)
