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

## 实现状态（2026-03-05）

Lency 当前是双链路并行：
- Rust 主编译器链路：功能更完整，作为稳定构建与验证主体。
- Lency 自举编译器链路（`lencyc/`）：按最小闭环持续补齐语法与语义能力。

### 自举阶段能力快照（2026-03-05）

- Lexer: 已支持 `int/float/scientific/string/char` 字面量。
- Parser: 已支持 `var/if/while/for/block/return/break/continue` 与 `call/member` 链，并已接入最小函数声明骨架（`int/string/bool/void/float` 起始）。
- Sema: 已支持最小 name resolution（undefined/duplicate/out-of-scope/shadowing）。
- Sema: 已支持 builtin 调用参数个数校验（arity）。
- Sema: 已支持用户函数最小 arity 校验（含先调用后声明）。
- Sema: 已支持用户函数签名类型校验（参数类型 + 返回类型，基础内建类型 token）。
- Sema: 已支持函数体最小 return 约束（禁止 `return` 空值，要求可达 value-return）。
- Sema: 已支持最小类型一致性检查（`int/bool/string/float`，覆盖赋值/一元/二元/逻辑）。
- Sema: 对 `arg_at/int_to_string/float_to_string/bool_to_string` 暂按 `unknown` 返回类型处理，以兼容现有 self-host runtime pointer-as-value 回归。
- Pipeline: 已打通 `Read -> Lex -> Parse -> Resolve -> Emit(AST/LIR)`。
- Tooling: `run_lency_checks.sh`、`lency_selfhost_build.sh`、`lency_selfhost_run.sh` 已接入回归闭环。

### 当前主线

当前开发优先级是语义增量（类型一致性与调用语义扩展），Parser 处于收尾阶段。
