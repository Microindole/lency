# Rust 母版目录

`crates/` 保存 Lency 的 Rust stage0 编译器、过渡后端和运行时。它是当前语言行为的参考实现，也负责把 `lencyc/` 构建为第一阶段编译器；开发主线仍是 Lency 自举实现。

## 编译流程

```text
lency_syntax -> lency_sema -> lency_monomorph -> lency_codegen
       |              |                              |
       +------ lency_diagnostics --------------------+
                                                      -> lency_runtime
lency_driver 负责串联阶段，lency_cli 提供命令行入口。
```

## 子 crate

| 目录 | 职责 |
|---|---|
| `lency_syntax/` | lexer、parser、AST |
| `lency_sema/` | 名称解析、类型与空安全检查 |
| `lency_monomorph/` | 泛型实例收集和单态化 |
| `lency_codegen/` | LLVM IR 和对象代码生成 |
| `lency_runtime/` | 生成程序调用的运行时 ABI |
| `lency_driver/` | 编译流水线和会话管理 |
| `lency_cli/` | `lencyc` 命令行和 LIR 过渡后端 |
| `lency_diagnostics/` | 共享诊断模型与终端输出 |
| `lency_ls/` | 语言服务器 |

## 修改边界

- Rust 母版已经进入稳定阶段；只有自举链路受阻或参考行为有缺陷时才优先修改。
- 语言契约以 [`docs/README.md`](../docs/README.md) 和 [`docs/design.md`](../docs/design.md) 为准。
- 修改 Rust 主链路后运行 `cargo run -p xtask -- auto-check`。
