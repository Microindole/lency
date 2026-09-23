# Lency 自举编译器

`lencyc/` 是用 Lency 编写的编译器实现，也是仓库当前开发主线。它已能处理自身使用的前端子集并发射项目 LIR，但 LIR 到 LLVM、链接和 runtime 仍依赖 Rust 母版。

## 当前流水线

```text
driver -> cli args -> lexer -> parser -> AST -> resolver -> LIR emitter
                                                        |
                                                        -> Rust LIR backend/runtime
```

## 子模块

| 目录 | 职责 |
|---|---|
| `cli/` | 自举编译器参数解析 |
| `syntax/` | token、lexer、AST 和 parser |
| `sema/` | 符号、作用域和语义解析 |
| `codegen/` | 项目 LIR 发射 |
| `driver/` | 编译器入口与阶段编排 |
| `runtime/` | selfhost runtime 预留边界；当前没有独立实现 |

## 开发边界

- 只移植自举直接需要且能形成最小闭环的能力。
- parser 接受、sema 可检查、LIR 可 lowering 和端到端可运行是不同成熟度。
- 不得以占位输出、未知类型降级或名称魔法伪造成功。
- 详细路线见 [`docs/development/bootstrap.md`](../docs/development/bootstrap.md)。

修改后运行 `cargo run -p xtask -- auto-check`；影响阶段构建或 LIR 时再运行 `cargo run -p xtask -- bootstrap-check`。
