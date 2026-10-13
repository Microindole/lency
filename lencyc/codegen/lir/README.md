# LIR Lowering 子模块

本目录扩展 `lencyc/codegen/lir.lcy`，处理需要独立上下文的 LIR 生成路径。

| 文件 | 职责 |
|---|---|
| `functions.lcy` | 函数签名、参数、局部值和返回 |
| `enum_support.lcy` | enum 声明、构造和 payload 表示 |
| `expr_call.lcy` | 普通函数及受支持成员调用 |
| `match_expr.lcy` | `match` 分支、模式与合流 |

生成的文本是 Rust LIR backend 的输入契约。新增或改变指令时必须同步 backend 测试，并运行 `cargo run -p xtask -- bootstrap-check` 验证 Stage2/Stage3 收敛。

函数和 impl 方法共用返回收尾规则：void 函数的可达末尾生成隐式 `ret`，非 void 函数的可达末尾产生错误，并由 `emit_lir_program` 清空输出。`current_block_terminated` 表示当前基本块已有 return/break/continue 终止指令，不表示函数内曾出现 return；if 分支分别维护状态，仅为未终止路径生成合流，while 出口恢复为可继续执行。顶层语句的合成 main 保留默认退出码 0，显式声明的 main 按普通函数检查。
