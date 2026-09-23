# Parser 子模块

本目录按语法类别拆分自举 parser；公共状态和顶层调度位于上一级 `parser.lcy`。

| 文件 | 职责 |
|---|---|
| `decl.lcy` | 函数、struct、enum、impl、import 和 extern 声明 |
| `stmt.lcy` | 局部变量、控制流和返回语句 |
| `expr.lcy` | 表达式、调用、成员访问、字面量和 `match` |

类型前置局部变量与函数声明共享起始形态，分流时必须保留清晰错误。`=>` 只用于 `match` 分支；不得恢复旧的 `T!` 或传播后缀语法。
