# AST 模块

本目录定义 Rust 母版 parser 产出的语法树。AST 只描述源码结构，不保存名称解析或 LLVM 信息。

| 文件 | 职责 |
|---|---|
| `expr.rs` | 表达式及模式相关节点 |
| `stmt.rs` | 声明和语句节点 |
| `types.rs` | 源码层类型引用 |
| `visitor.rs` | AST 访问接口 |
| `mod.rs` | 节点导出 |

新增或删除节点时，必须检查 parser、sema、monomorph 和 codegen 的所有遍历路径。内建函数保持普通 `Call` 节点，不得恢复按名称区分的专用表达式。
