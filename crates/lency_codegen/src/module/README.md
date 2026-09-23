# 模块 Codegen

本目录建立 LLVM module 级声明，然后协调函数体生成。

| 文件 | 职责 |
|---|---|
| `mod.rs` | module 生成入口和阶段顺序 |
| `types.rs` | struct、enum 等用户类型声明 |
| `functions.rs` | 函数签名、全局项和函数体调度 |

类型和函数通常需要先声明再生成主体，以支持递归与跨函数引用。未知用户类型不得静默降级为任意指针 ABI。
