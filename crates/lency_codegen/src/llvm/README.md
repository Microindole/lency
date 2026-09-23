# LLVM 辅助层

本目录封装较底层的 LLVM 构建操作，减少上层 lowering 对 `inkwell` 细节的重复处理。

| 文件 | 职责 |
|---|---|
| `context.rs` | LLVM 上下文与 module 生命周期 |
| `builder.rs` | 指令构建辅助 |
| `translator.rs` | Lency 类型和值到 LLVM 表示的转换 |

这里只提供 LLVM 机制，不决定语言语义。类型合法性和用户可见诊断应在 sema 或上层 codegen 边界完成。
