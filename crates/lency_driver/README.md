# lency_driver

编译器的阶段编排层。它连接语法、语义、单态化、codegen 与诊断，但不拥有各阶段的具体规则。

## 结构

| 文件 | 职责 |
|---|---|
| `src/lib.rs` | 对外编译入口和阶段组合 |
| `src/error.rs` | driver 层错误封装 |
| `examples/` | driver API 示例 |

新增编译阶段时，应明确输入、输出和诊断归属，避免把 parser、sema 或 codegen 细节塞入 driver。

编译入口与导入解析共用换行语句边界规则。多文件语义失败保留源码映射，诊断使用原始文件和局部字节范围；成功路径把同一映射交给 codegen 生成源码调用帧。
