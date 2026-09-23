# lency_cli

提供 `lencyc` 可执行文件和用户命令入口，并承载当前自举流程仍使用的 LIR 到 LLVM 过渡后端。

## 结构

| 路径 | 职责 |
|---|---|
| `src/main.rs` | 进程入口和命令分派 |
| `src/cli.rs` | 命令行参数模型 |
| `src/commands/` | `build`、`check`、`compile`、`run`、REPL |
| `src/lir_backend/` | 读取 selfhost LIR 并完成本机构建 |
| `src/path_utils.rs` | 路径处理辅助函数 |

CLI 只负责交互与编排。语言规则应落在对应编译阶段；LIR backend 是混合自举的过渡边界，不代表 selfhost 已拥有独立本机后端。
