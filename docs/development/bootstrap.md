# 自举路线

## 角色划分

仓库中的 Rust 实现是完整母体编译器，当前不以继续扩展它为主线。Lency 实现位于 `lencyc/`，目标是逐步替代母体完成编译器自身构建。

```text
Rust 母体（stage0）
    -> 编译 lencyc 源码
Lency 编译器（stage1）
    -> 编译自身并发射 LIR
Lency 编译器（stage2）
    -> 再次编译自身
Lency 编译器（stage3）
    -> 比较 stage2/stage3 的 LIR
```

stage2 与 stage3 的 LIR 完全一致，表示当前输入和工具链下达到 LIR 层收敛。二进制完全一致仍是可选的严格检查，不作为默认门槛。

## 当前是混合自举

现有链路还不是完全独立的 native selfhost：

1. Rust 母体把 `lencyc/driver/main.lcy` 构建为 stage1。
2. stage1 读取 Lency 编译器源码并发射 LIR。
3. Rust 母体的 LIR backend 把这份 LIR 构建为 stage2。
4. stage2 再次发射 LIR，Rust 母体再把它构建为 stage3。

因此，当前已经验证的是“Lency 前端、语义子集和 LIR emitter 可以处理自身，且输出能够收敛”。尚未验证的是“Lency 编译器拥有独立的 LLVM/机器码后端并能完全脱离 Rust 母体”。

另外，收敛只说明两次输出一致，不自动证明所有输入都被正确处理。selfhost emitter 已禁止用 `expr_unknown` 等占位值伪造成功：无法 lowering 的表达式、语句或运算符会产生显式诊断，Rust LIR backend 也会拒绝遗留的 `expr_unknown`。函数签名只接受基础类型、`Vec`、nullable 类型和当前编译单元中已声明的 struct/enum；未知 ABI 类型会终止 LIR 生成，不再默认降级为指针。

## 当前最小流水线

```text
Read -> Lex -> Parse -> Resolve -> Emit AST/LIR
```

职责目录：

| 路径 | 职责 |
|---|---|
| `lencyc/syntax/` | token、lexer、AST、parser |
| `lencyc/sema/` | 名称解析、作用域与最小类型约束 |
| `lencyc/codegen/lir/` | selfhost LIR 发射 |
| `lencyc/driver/` | 编译器入口与管线 |
| `lib/` | 自举可用的标准库源码 |
| `tests/example/selfhost/` | parser/sema 自举回归 |
| `tests/example/lir/` | LIR 文本与构建回归 |
| `tests/example/runtime/` | 生成程序的端到端回归 |

Rust 侧的 LIR backend 和 runtime 仍是过渡基础设施。普通 Lency 程序目前也应使用 Rust 母体。只有自举链路需要新的承载能力或参考行为存在缺陷时，才应修改 `crates/`。

## 工作顺序

1. 用最小 `.lcy` 用例描述缺口。
2. parser 行为有变化时，补 parser 正例与负例。
3. 语义行为有变化时，优先补 `tests/example/selfhost/driver/steps/` 回归。
4. lowering 或 runtime 行为有变化时，补 `tests/example/runtime/` 端到端回归。
5. 运行 `cargo run -p xtask -- check-lency`。
6. 影响自举编译器自身构建时，再运行 `cargo run -p xtask -- bootstrap-check`。

不要为了补一个后端缺口，顺便扩大语言表面；也不要只让 parser 接受新语法而没有语义或运行路径。

## Rust 能力进入 selfhost 的准入规则

Rust 母体中的能力不应整批移植。只有满足以下至少一项时，才进入近期 selfhost 范围：

1. `lencyc/` 编译自身已经直接使用；
2. 是正确表达编译器核心数据结构或控制流所必需；
3. 能移除当前自举路径上的硬编码、占位输出或静默错误；
4. 是下一阶段独立构建不可缺少的后端能力。

进入 selfhost 后必须形成最小闭环：parser（如需要）→ sema → LIR lowering → runtime/backend → 回归。只有 parser 或 sema 支持的能力，不应继续扩展表面语法。

当前应优先：

- 让编译器自身已经使用的函数、struct、enum/match、字符串、Vec、import 和普通结果数据路径稳定；
- 将剩余的宽松回退变成明确错误或真实 lowering，并保持 LIR backend 拒绝占位指令；
- 缩小 stage2/stage3 对 Rust 母体后端的依赖。

当前可以推迟：

- 与编译器自举无直接关系的完整容器 API；
- 闭包、完整 trait 体系和面向普通应用的高级泛型能力；
- REPL、语言服务器和扩展性功能的 selfhost 重写；
- 仅为追求 Rust 母体功能对齐而增加的语法。

## 常用命令

```powershell
# Lency 主线检查
cargo run -p xtask -- check-lency

# stage1 -> stage2 -> stage3 收敛
cargo run -p xtask -- bootstrap-check

# 使用 selfhost 编译或运行一个文件
cargo run -p xtask -- selfhost-build <input.lcy> -o <name>
cargo run -p xtask -- selfhost-run <input.lcy>
```

`auto-check` 根据未提交文件选择范围。干净工作区会回退到 `check-lency`；它不能代替首次搭建环境时显式执行全部需要的检查。
