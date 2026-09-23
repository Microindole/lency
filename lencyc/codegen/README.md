# Codegen 模块

本目录把 selfhost 已解析并检查的 AST 发射为项目自定义 LIR。

| 路径 | 职责 |
|---|---|
| `lir.lcy` | LIR emitter 状态、基础表达式和语句调度 |
| `lir/` | 函数、enum、调用和 `match` 的 lowering |

输出由 Rust 母版中的 LIR backend 转换为 LLVM 和本机程序。这里不应发射 `expr_unknown` 等占位指令，也不能把未知用户类型默认当作指针 ABI。
