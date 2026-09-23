# lency_diagnostics

提供 Rust 母版各阶段共享的诊断数据结构和终端呈现。

## 结构

| 文件 | 职责 |
|---|---|
| `src/diagnostic.rs` | 诊断主体、标签和附加信息 |
| `src/level.rs` | 错误级别 |
| `src/span.rs` | 源码位置与范围 |
| `src/sink.rs` | 诊断收集接口 |
| `src/emitter.rs` | 终端格式化输出 |
| `src/lib.rs` | 公共导出 |

诊断文本应说明实际错误和源码位置；不要在调用方重复实现颜色、布局或 span 格式化。
