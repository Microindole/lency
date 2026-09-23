# LIR Lowering 子模块

本目录扩展 `lencyc/codegen/lir.lcy`，处理需要独立上下文的 LIR 生成路径。

| 文件 | 职责 |
|---|---|
| `functions.lcy` | 函数签名、参数、局部值和返回 |
| `enum_support.lcy` | enum 声明、构造和 payload 表示 |
| `expr_call.lcy` | 普通函数及受支持成员调用 |
| `match_expr.lcy` | `match` 分支、模式与合流 |

生成的文本是 Rust LIR backend 的输入契约。新增或改变指令时必须同步 backend 测试，并运行 `cargo run -p xtask -- bootstrap-check` 验证 Stage2/Stage3 收敛。
