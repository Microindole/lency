# lency_runtime

提供生成程序调用的 Rust 运行时和稳定 C ABI。它同时构建为 `rlib` 与动态库。

## 结构

| 路径 | 职责 |
|---|---|
| `src/lib.rs` | ABI 导出、基础内存和容器入口 |
| `src/string.rs` | 字符串分配、查询与转换 |
| `src/file.rs` | 文件系统运行时操作 |
| `src/enum_value.rs` | enum payload 运行时表示 |
| `src/hashmap.rs` | 整数键 HashMap |
| `src/hashmap_string.rs` | 字符串键 HashMap |

## 边界

codegen 中的声明必须与这里的符号名、参数、返回类型和所有权约定完全一致。修改 ABI 时必须同时检查 codegen、LIR backend 和端到端运行测试。
