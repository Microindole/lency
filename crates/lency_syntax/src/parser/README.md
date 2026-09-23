# Parser 模块

本目录把 lexer token 流解析为 AST，并负责语法错误恢复。

| 路径 | 职责 |
|---|---|
| `mod.rs` | parser 状态、公共入口和顶层流程 |
| `decl.rs` | 函数、类型、impl、import 等声明 |
| `stmt.rs` | 局部变量和控制流语句 |
| `expr/` | 原子、后缀、一元和二元表达式 |
| `pattern.rs` | `match` 模式 |
| `helpers.rs` | token 消费和公共解析辅助 |

语法接受不等于功能可用。新增语法必须同时确认 sema、lowering 和回归范围；`=>` 只用于 `match` 分支。
