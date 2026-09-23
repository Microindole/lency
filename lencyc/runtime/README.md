# Runtime 边界

本目录预留给未来用 Lency 实现或管理的 selfhost runtime，目前没有独立运行时代码。

现阶段生成程序仍链接 `crates/lency_runtime`，LIR 到 LLVM 也仍由 Rust 母版的过渡后端完成。这里的存在不表示已经脱离 Rust runtime。

引入实现前必须先明确 ABI、内存所有权和与 Rust runtime 的迁移顺序，并为 selfhost 运行路径增加端到端测试。
