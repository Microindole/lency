# Driver 模块

本目录提供 Lency 自举编译器入口并串联当前编译阶段。

| 文件 | 职责 |
|---|---|
| `main.lcy` | 参数处理、读取源码、lex、parse、resolve 和输出 |
| `pipeline_sample.lcy` | 最小管线输入，用于自举阶段回归 |

当前流程为 `Read -> Lex -> Parse -> Resolve -> Emit AST/LIR`。driver 只负责阶段编排；具体语言规则必须留在 syntax、sema 或 codegen。
