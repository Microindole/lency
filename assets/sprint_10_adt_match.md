# Sprint 10 设计文档: ADTs 与 模式匹配 (Detailed Spec)

## 1. 目标
引入代数数据类型 (ADTs) 和模式匹配，构建 Beryl 的现代类型系统核心。本阶段重点解决**内存布局**、**泛型单态化**及**模式匹配编译**三大技术难点。

## 2. 语法规范

### 2.1 枚举 (Enum) 定义
支持 C 风格无值枚举和 Rust 风格带值枚举。

```beryl
// 1. 无值枚举 (C-like)
enum Status {
    Idle,
    Running,
    Done
}

// 2. 带值枚举 (Tagged Union)
enum Shape {
    Circle(float),             // Tuple Variant
    Rect(float, float),
    Unit                       // Unit Variant inside Tagged Union
}

// 3. 泛型枚举
enum Option<T> {
    Some(T),
    None
}
```

> **注意**: 递归枚举（如 `enum List { Cons(int, List), Nil }`）必须通过引用或指针类型（如 `Box<List>`）打破循环，否则会导致无限大小。编译器需在 Sema 阶段检测并报错 "Recursive type has infinite size"。

### 2.2 模式匹配 (Match Expression)
`match` 是一个表达式，必须有返回值。

```beryl
float area(Shape s) {
    return match s {
        case Circle(r) => 3.14 * r * r,
        case Rect(w, h) => w * h,
        case Unit => 0.0,
        case _ => 0.0 // Wildcard
    }
}
```

## 3. 内存布局与 ABI (Codegen)

### 3.1 布局策略 (Tagged Union Layout)
为了兼容 C ABI 并优化内存，我们采用 **Discriminant + Union** 布局。

逻辑结构:
```rust
struct EnumLayout {
    tag: u8,       // 判别式 (Discriminant)
    padding: ...   // 对齐填充
    payload: Union // 最大变体数据
}
```

#### LLVM IR 表示
LLVM 没有原生 Union 类型。我们使用 **字节数组 (`[Size x i8]`)** 来预留 Payload 空间。

**算法: 计算布局参数**
对于 `enum E { V1(T1), V2(T2), ... }`:
1.  **Payload Size**: `S = max(sizeof(T1), sizeof(T2), ...)`
2.  **Payload Align**: `A = max(alignof(T1), alignof(T2), ...)`
3.  **Tag Size**: 默认 `i8` (支持<256个变体)。
4.  **Struct Layout**: `{ i8, [padding], [S x i8] }`。

**示例: `Option<int>` (int = i32)**
*   `Some(i32)`: size=4, align=4
*   `None`: size=0, align=1
*   **Result**: 
    *   Payload Size = 4, Align = 4.
    *   Tag = `i8`.
    *   LLVM Type: `<{ i8, [3 x i8], [4 x i8] }>` (Packed struct with manual padding to ensure align 4 for payload? No, better use LLVM struct alignment).
    *   **Better LLVM Implementation**: 
        创建一个 `Opaque Union Type` 作为存储类型: `{ i8, [4 x i8] }`，并将该 Struct 的 Alignment 设置为 4。

#### 泛型布局
`Option<T>` 不是一个具体的 LLVM 类型。只有实例化后（如 `Option<int>`, `Option<string>`）才会生成具体的 LLVM Type。Codegen 阶段需根据 `T` 的具体大小动态生成布局。

### 3.2 构造器与访问
*   **构造 (Construction)**:
    *   `Shape::Circle(5.0)`: 
        1. `alloca` Enum Storage。
        2. Store Tag = 0。
        3. Bitcast Storage 指针 -> `{ i8, float }*`。
        4. Store Payload (5.0)。
*   **访问 (Access)**:
    *   仅允许通过 `match` 解构访问，禁止直接字段访问（不安全）。

## 4. 模式匹配编译策略

### 4.1 简单匹配 (Single Level)
对于不含嵌套模式的匹配，直接编译为 LLVM `switch` 指令。

**Source:**
```beryl
match s {
    case Circle(r) => ...
    case Rect(w, h) => ...
}
```

**LLVM IR Pseudocode:**
```llvm
%tag_ptr = getelementptr %s, 0, 0
%tag = load i8, %tag_ptr
switch i8 %tag, label %default [
    i8 0, label %case_circle
    i8 1, label %case_rect
]

case_circle:
    %cast_ptr = bitcast %s* to { i8, float }*
    %r_ptr = getelementptr %cast_ptr, 0, 1
    %r = load float, %r_ptr
    ; ... body ...
    br label %end

case_rect:
    %cast_ptr = bitcast %s* to { i8, {float, float} }*
    ; ... destructure ...
    br label %end
```

### 4.2 嵌套匹配 (Nested Patterns) - *Phase 2*
支持 `case Some(Ok(val)) => ...`。
这需要将其展开为**决策树 (Decision Tree)**。
对于 Sprint 10，我们优先实现单层匹配。嵌套匹配如遇到，暂时报错或作为进阶目标。

### 4.3 穷尽性检查 (Exhaustiveness)
Sema 阶段必须验证覆盖率。
算法：
1.  收集 `match` 中的所有 Tag 集合 `S_match`.
2.  获取 Enum 的完整 Tag 集合 `S_enum`.
3.  如果 `Has_Wildcard (_)`: 安全。
4.  否则，断言 `S_match == S_enum`。如果不等，报错 "Pattern not exhaustive. Missing variants: [Diff]"。

### 4.4 与空安全系统 (Null Safety) 的集成
Beryl 的空安全系统 (Sprint 6) 与 ADTs 完美共存：

1.  **非空默认规则**: 枚举类型 (`Enum`) 默认为非空。
    *   `var e: Shape = null` ❌ 编译错误。
    *   `var e: Shape? = null` ✅ 允许。

2.  **`Option<T>` vs `T?`**:
    *   `T?` 是语言内置的空指针类型，零开销 (Zero-Cost)，对应 C 指针语义。
    *   `Option<T>` 是标准库定义的 ADT，用于更高级的功能模式（如 Monadic 操作），或当需要区分 "无数据" 与 "未初始化" 等语义时。
    *   **建议**: 在 FFI 或高性能场景使用 `T?`，在纯 Beryl逻辑中使用 `Option<T>` 或 `T?` 均可。

3.  **对可空类型的模式匹配**:
    模式匹配天然支持 `T?` 的解包：
    ```beryl
    float safe_area(Shape? s) {
        return match s {
            case null => 0.0,
            case val => area(val) // Smart Cast: 'val' is inferred as Shape (non-null)
        }
    }
    ```
    Sema 需确保 `case null` 被处理，或者有 wildcard `_`，否则穷尽性检查报错。

## 5. 实现阶段规划

### Phase 1: 核心定义 (Core Defs)
*   **Syntax**: `EnumDef`, `EnumVariant`.
*   **Sema**: `Type::Enum(name, generics)`.
*   **Monomorphization**: 扩展 `Monomorphizer` 以处理 Enum 实例化。

### Phase 2: 后端支持 (Backend)
*   **Type Layout**: 实现上述布局算法。
*   **Constructors**: 实现 `Decl::EnumVariant` 的代码生成（作为伪函数）。

### Phase 3: 匹配实现 (Match)
*   Parser 支持 `case Variant(...)`.
*   Sema 绑定变量并推导类型。
*   Codegen 生成 Switch。

## 6. 测试用例验证
*   `tests/integration/adt/enum_size.brl`: 验证不同 payload 大小的 Enum 的 `sizeof`（如果有 sizeof 运算符）或内存表现。
*   `tests/integration/adt/option_monomorph.brl`: 验证 `Option<int>` 和 `Option<bool>` 是否生成正确的不同布局。
*   `tests/integration/adt/match_exhaust.brl`: 验证穷尽性检查报错。
