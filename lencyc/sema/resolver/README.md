# Resolver 子模块

本目录扩展 `lencyc/sema/resolver.lcy`，完成自举前端的声明预载、类型传播和表达式/语句检查。

| 路径 | 职责 |
|---|---|
| `program.lcy` | 顶层声明预载顺序 |
| `decl_stmt.lcy`、`decl_extra.lcy` | 声明解析与实现检查 |
| `decl_std_auto.lcy` | 从标准库源码提取可见签名 |
| `stmt.lcy` | 局部符号、赋值与控制流语义 |
| `return_flow.lcy` | 函数返回覆盖检查 |
| `type_names.lcy`、`type_system.lcy` | 类型名称和兼容性规则 |
| `core/` | 符号表、作用域与 enum 签名辅助 |
| `expr/` | 调用、模式和表达式递归检查 |

`core/` 与 `expr/` 是为控制文件体积形成的实现分组，不是新的编译阶段。未知类型、未知调用和 enum payload 不一致应产生诊断，不得依赖 codegen 兜底。
