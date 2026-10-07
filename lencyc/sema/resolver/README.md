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

`expr/member.lcy` 按接收者类型查找 impl 方法签名，供本地和导入声明共用；它同时维护自举子集中的 Vec/字符串方法契约。成员调用检查参数数量与类型，并传播完整返回类型名，不能退回到同名全局函数冒充 struct 方法。字符串/Vec 的扩展函数仅在首参数类型与接收者一致时可用。方法体中的 `this` 带有所属类型；泛型 impl 实例化尚不属于已完成子集。

`core/` 与 `expr/` 是为控制文件体积形成的实现分组，不是新的编译阶段。未知类型、未知调用和 enum payload 不一致应产生诊断，不得依赖 codegen 兜底。
