# Syntax 模块

本目录是 Lency 自举编译器的前端语法层，将源码转换为 AST。

## 结构

| 路径 | 职责 |
|---|---|
| `token.lcy` | token 类型和源码位置 |
| `lexer.lcy` | 字符扫描与 token 生成 |
| `ast.lcy`、`ast/` | AST 数据结构、构造器和打印 |
| `parser.lcy`、`parser/` | 声明、语句和表达式解析 |

## 数据流

```text
source string -> Lexer -> Vec<Token> -> Parser -> Program
```

本层不赋予 `Result`、内建函数或其他用户名称特殊语义。语法变化必须覆盖正例、负例和错误恢复，并确认 sema/LIR 是否支持。
