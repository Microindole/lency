# lency_syntax

Rust 母版的语法前端，将 Lency 源码转换为不带语义结论的 AST。

## 结构

| 路径 | 职责 |
|---|---|
| `src/lexer.rs` | token 定义、词法分析和源码跨度 |
| `src/parser/` | 声明、语句、表达式和模式解析 |
| `src/ast/` | AST 节点、类型引用和访问器 |
| `src/lib.rs` | 模块导出和前端回归入口 |
| `examples/` | 独立语法示例 |

## 边界

这里不执行名称解析、类型推导或 lowering。内建函数在语法层也必须表现为普通调用，不得按名称生成专用 AST。

语法变化必须同步语法测试和 `docs/` 中相应语言文档，并通过 `cargo run -p xtask -- auto-check`。
