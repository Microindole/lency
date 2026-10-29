# 语句 Codegen

本目录负责语句和结构化控制流的 LLVM lowering。

| 路径 | 职责 |
|---|---|
| `mod.rs` | 变量、返回、表达式语句和控制流分派 |
| `control_flow/conditional.rs` | `if` 分支 |
| `control_flow/loops.rs` | `while` 等循环 |
| `control_flow/for_in.rs` | `for-in` lowering |

控制流必须维护正确的基本块终止状态以及 `break`、`continue`、`return` 目标。语义非法路径应在 sema 阶段拒绝。

块和循环头保存、恢复局部名称绑定，但不撤销对外层变量存储的赋值。终止块不再追加语句；条件两侧均终止时移除无前驱合流块，并保持生成位置处于终止状态。
