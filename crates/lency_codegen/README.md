# lency_codegen

Rust 母版的 LLVM codegen，将已完成语义检查和单态化的 AST 降低为 LLVM IR、对象文件或可执行文件所需模块。

## 结构

| 路径 | 职责 |
|---|---|
| `src/context.rs` | codegen 状态与共享上下文 |
| `src/module/` | 模块、类型和函数声明阶段 |
| `src/function.rs` | 函数体生成入口 |
| `src/expr/` | 表达式 lowering |
| `src/stmt/` | 语句和控制流 lowering |
| `src/llvm/` | LLVM 构建辅助层 |
| `src/runtime.rs` | runtime ABI 声明 |
| `src/layout.rs`、`src/types.rs` | 值布局和 LLVM 类型映射 |

## 边界

codegen 不应修补前端本应拒绝的类型错误。运行时原语的专用 lowering 必须集中、可测试，并保持语法层为普通调用。
