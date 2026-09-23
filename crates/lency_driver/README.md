# lency_driver

编译器的阶段编排层。它连接语法、语义、单态化、codegen 与诊断，但不拥有各阶段的具体规则。

## 结构

| 文件 | 职责 |
|---|---|
| `src/lib.rs` | 对外编译入口和阶段组合 |
| `src/pipeline.rs` | 编译流程与阶段数据传递 |
| `src/session.rs` | 单次编译会话状态 |
| `src/error.rs` | driver 层错误封装 |
| `examples/` | driver API 示例 |

新增编译阶段时，应明确输入、输出和诊断归属，避免把 parser、sema 或 codegen 细节塞入 driver。
