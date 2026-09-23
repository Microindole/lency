# AST 子模块

本目录补充 `lencyc/syntax/ast.lcy` 中的 AST 定义和行为。

| 文件 | 职责 |
|---|---|
| `decl.lcy` | 声明节点构造和声明载荷转换 |
| `stmt.lcy` | 语句节点构造，包括显式类型局部变量 |
| `expr.lcy` | 表达式节点构造 |
| `visitor.lcy` | AST 访问辅助 |
| `printer.lcy` | 稳定的 AST 文本输出 |

`ast.lcy` 保存共享节点结构和 kind 常量，本目录文件通过 `impl` 扩展行为。新增字段时必须更新所有构造器、printer、resolver 和 LIR emitter。
