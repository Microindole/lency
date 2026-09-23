# lency_ls

Lency 的语言服务器，复用 Rust 母版的 syntax、sema 和 diagnostics，为编辑器提供 LSP 能力。

## 结构

| 路径 | 职责 |
|---|---|
| `src/main.rs` | LSP 进程入口 |
| `src/lib.rs` | server 状态、请求处理和编译器桥接 |
| `tests/` | 协议与行为回归 |

语言服务器不定义新的语言语义。解析和检查行为应来自编译器 crate；编辑器侧设计与状态记录在 `editors/docs/`。
