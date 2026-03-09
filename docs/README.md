# Lency 语言文档

欢迎使用 Lency。

## 快速开始

```lency
int main() {
    print("Hello, Lency!")
    return 0
}
```

## 文档目录

### 基础
- [变量与类型](./basics/variables.md)
- [函数](./basics/functions.md)
- [控制流](./basics/control-flow.md)

### 类型系统
- [基础类型总览](./types/primitives.md)
- [Bool](./types/bool.md)
- [Float](./types/float.md)
- [结构体](./types/structs.md)
- [枚举](./types/enums.md)
- [Null 安全](./types/null-safety.md)

### 标准库
- [Vec](./stdlib/vec.md)
- [字符串操作](./stdlib/string.md)
- [文件 I/O](./stdlib/file-io.md)
- [HashMap](./stdlib/hashmap.md)

### 工具链
- [脚本指南](./tools/scripts.md)

---

## 实现状态（2026-03-09）

Lency 当前是双链路并行：
- Rust 主编译器链路：功能更完整，作为稳定构建与验证主体。
- Lency 自举编译器链路（`lencyc/`）：按最小闭环持续补齐语法与语义能力。

### 自举阶段能力快照（2026-03-09）

- Lexer: 已支持 `int/float/scientific/string/char/null` 字面量。
- Parser: 已支持 `var/if/while/for/block/return/break/continue` 与 `call/member` 链，并已接入 `function/struct/impl/import/extern/enum/match` 声明与表达式子集（含 `import std.*` 通配导入语法）。
- Sema: 已支持最小 name resolution（undefined/duplicate/out-of-scope/shadowing）。
- Sema: 已支持 builtin 调用参数个数校验（arity）。
- Sema: 已支持用户函数最小 arity 校验（含先调用后声明）。
- Sema: 已支持用户函数签名类型校验（参数类型 + 返回类型，基础内建类型 token）。
- Sema: 已支持函数体最小 return 约束（禁止 `return` 空值，要求可达 value-return）。
- Sema: 已支持最小类型一致性检查（`int/bool/string/float`，覆盖赋值/一元/二元/逻辑）。
- Sema: 已支持 `enum + match` 语义第一版（重复 pattern、未知 variant、穷尽性检查）。
- Sema: 已支持 `match` payload 绑定第一版（`Text(v)` / `Pair(a,b)`），绑定变量参与 arm 内类型检查。
- Sema: 已支持 `Result` builtin enum（`Ok/Err`）的构造与 `match` 校验。
- Sema: 已支持 `null` 最小语义（字面量 + 基础类型约束检查）。
- Sema: 已支持 enum 类型流扩展到函数返回、`match` 中间表达式与赋值链路。
- Sema: import 语义第一版已支持非 `std.*` 模块文件加载与声明符号导入。
- Sema: `std.*` 已切到“模块源码签名自动导入”（递归 `import std.*`），不再依赖模块白名单最小符号预加载。
- Sema: 已支持 `import std.*` 全量标准模块签名自动预加载；非 `std.*` 的通配导入会报错，避免静默误解语义。
- Sema: 对 `arg_at/int_to_string/float_to_string/bool_to_string` 暂按 `unknown` 返回类型处理，以兼容现有 self-host runtime pointer-as-value 回归。
- Sema: 已支持 nullable 签名语义（`int?/string?/bool?/float?` + 自定义 `Type?`），自定义可空类型不再走 `TYPE_UNKNOWN` 兼容放行。
- Backend: Rust LIR backend member lowering 已改为“intrinsic 映射 + 通用 fallback”统一路径（含 `to_string/len/trim/substr/split/format/join`）。
- Pipeline: 已打通 `Read -> Lex -> Parse -> Resolve -> Emit(AST/LIR)`。
- Tooling: 规范入口统一为 `cargo run -p xtask -- check-rust` 与 `cargo run -p xtask -- check-lency`（平台脚本仅为包装）。

### 自举编译器内部结构快照（2026-03-08）

- AST: 声明数据已从 `Stmt` 散落字段收敛为 `stmt.decl` payload，新增声明特性不再需要修改 `Stmt` 结构体字段列表。
- Resolver: 声明语义路径已切到 `Decl` 视图处理，`resolve_stmt` 仅负责语句分派。
- AST Printer: expr 打印分派已引入 Visitor 试点（低风险路径验证）。

### 当前主线

当前开发优先级是语义增量（类型一致性与调用语义扩展），Parser 处于收尾阶段。
