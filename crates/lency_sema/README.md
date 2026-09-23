# lency_sema

Rust 母版的语义分析层，接收 `lency_syntax` AST，产出经过名称和类型验证、可供单态化与 codegen 使用的程序。

## 结构

| 路径 | 职责 |
|---|---|
| `src/resolver/` | 名称、导入、声明和作用域解析 |
| `src/type_infer/` | 表达式类型推导 |
| `src/type_check/` | 声明、语句和调用的类型约束 |
| `src/null_safety/` | nullable 流分析 |
| `src/types/` | 类型信息与注册表 |
| `src/operators/` | 运算符类型规则 |
| `src/scope/` | 符号作用域栈 |
| `src/error.rs` | 语义错误模型 |

## 边界

强静态错误应在本层拒绝，不得依赖 codegen 猜测类型。`Result`、`Ok`、`Err` 等普通用户名称没有特权；内建函数以全局符号参与名称与类型检查。
