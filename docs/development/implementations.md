# 实现分层与成熟度

Lency 仓库包含一个参考实现和一个正在自举的实现。讨论“是否支持某项能力”时，必须说明属于哪一层。

## 两套实现

### Rust 母体（stage0）

位于 `crates/`，是当前普通 Lency 程序使用的编译器，也是语言行为的参考实现。它提供：

- lexer、parser、AST、语义分析与诊断；
- 泛型单态化；
- LLVM codegen 与 runtime；
- `compile`、`run`、`check`、`build` 和实验性 REPL；
- 数组/Vec、struct/方法、enum/match、泛型、trait、nullable、模块和标准库等集成回归。

这里的“参考实现”不表示没有缺陷，而是表示当前语言示例、普通程序和 selfhost stage0 都以它为准。

### Lency 自举编译器

位于 `lencyc/`，当前能够读取源码、词法分析、解析、执行一部分语义检查，并发射 AST 文本或项目自定义 LIR。

它已经不是空壳，但仍是受限实现：

- parser/sema 支持范围大于 LIR lowering 范围；
- LIR emitter 只覆盖自举和现有 runtime 回归需要的子集；
- 尚未自己完成 LIR 到 LLVM/机器码以及最终链接；
- `xtask selfhost-build` 和 bootstrap 后续阶段仍调用 Rust 母体处理 LIR；
- 未覆盖的表达式或语句路径中仍存在占位输出，因此“成功发射”不能单独证明语义正确。

## 能力矩阵

| 能力 | Rust 母体 | selfhost 前端 | selfhost 端到端 |
|---|---|---|---|
| 基础表达式、变量、控制流 | 可用 | 已覆盖主要子集 | 已有 LIR/runtime 回归 |
| 普通函数 | 可用 | 解析与基础签名检查 | 最小调用、参数、返回已回归 |
| struct | 可用 | 声明、字段与基础语义 | 仅 non-generic 字段读写、参数和返回 |
| enum / match | 可用 | payload、嵌套模式、guard 等已有检查 | 现有递归 payload/runtime 用例可运行 |
| import / std 签名 | 可用 | 已有源码签名导入 | 仅自举所需路径经过验证 |
| nullable / 普通 enum 结果 | nullable 与 enum 可用；无内建 Result | selfhost 仍有旧 Result 兼容语义 | 仅已有用例覆盖的路径 |
| 泛型 | 有单态化与集成测试 | 有语法入口 | 尚非稳定子集 |
| trait / 完整 impl method | 有集成测试 | 不完整 | 尚非稳定子集 |
| 闭包、完整集合/标准库 | 有相应实现或测试 | 不完整 | 尚非稳定子集 |
| LLVM/本机代码生成 | 可用 | 不负责 | 仍依赖 Rust 母体 |
| 独立编译并链接自身 | 作为 stage0 可完成 | 尚不能独立完成 | 当前是混合自举 |

矩阵只描述能力层级，不使用主观完成百分比。更细的事实以对应源码和回归测试为准。

## 文档用语

- **可解析**：parser 能构造 AST，不代表语义或后端可用。
- **可检查**：resolver/sema 能处理，不代表能生成代码。
- **可 lowering**：selfhost 能发射对应 LIR，不代表已经脱离 Rust 母体。
- **端到端**：存在 `.lcy -> selfhost LIR -> Rust LIR backend/runtime -> executable` 回归。
- **独立自举**：Lency 实现能在不借用 Rust 母体后端的情况下构建下一阶段；当前尚未达到。

## 默认选择

- 编写和运行普通 Lency 程序：使用 Rust 母体 `lencyc`。
- 开发自举编译器：使用 `xtask check-lency`、`selfhost-build`、`selfhost-run`。
- 验证阶段收敛：使用 `xtask bootstrap-check`，同时记住它验证的是当前混合链路的 LIR 收敛，不是完整功能等价证明。
