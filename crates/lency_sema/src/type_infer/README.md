# Type Infer 模块

本目录推导表达式和控制流结果类型，供 type check 与后续阶段使用。

| 路径 | 职责 |
|---|---|
| `mod.rs` | 推导器状态和表达式分派 |
| `literal.rs` | 字面量类型 |
| `operators.rs` | 一元和二元表达式结果 |
| `call.rs` | 调用表达式适配与统一检查入口 |
| `adt.rs` | struct、enum 等代数数据类型 |
| `access/` | 变量、成员和索引访问 |
| `control.rs` | 分支和控制流表达式 |
| `tests.rs` | 局部推导回归 |

`call.rs` 不维护第二套宽松调用规则；调用约束统一委托给 type checker，避免嵌套调用绕过强静态检查。
