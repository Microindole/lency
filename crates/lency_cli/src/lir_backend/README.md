# LIR 过渡后端

本目录读取 selfhost 发射的项目自定义 LIR，并借助 Rust/LLVM 工具链完成本机构建。

| 路径 | 职责 |
|---|---|
| `mod.rs` | LIR backend 入口 |
| `emitter.rs` | LIR 文本解析和 LLVM 输出协调 |
| `compile/` | 调用、成员访问和编译辅助 |
| `tests.rs` | LIR 接受与拒绝回归 |

这是当前混合自举的一部分。未知指令、未知 ABI 类型和占位表达式必须显式失败，不能为了让阶段继续而猜测 lowering。
