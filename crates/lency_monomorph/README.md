# lency_monomorph

负责 Rust 母版中的泛型单态化，在 codegen 前把实际使用的泛型声明改写为具体实例。

## 结构

| 路径 | 职责 |
|---|---|
| `src/collector.rs` | 收集需要实例化的泛型使用点 |
| `src/specializer/` | 按类型实参专门化声明、语句和表达式 |
| `src/rewriter.rs` | 重写调用和类型引用 |
| `src/mangling.rs` | 生成稳定的实例符号名 |
| `src/lib.rs` | 单态化入口与模块导出 |

本 crate 只处理泛型实例，不负责名称解析或 LLVM 布局。新增 AST 节点时应同步检查 collector、rewriter 和 specializer 是否需要遍历它。
