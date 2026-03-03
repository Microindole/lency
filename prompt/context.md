# Lency 项目上下文入口

## 0. 最高准则
- 语言与设计哲学：`assets/Lency.txt`、`assets/design_spec.md`（冲突时以这两个文件为准）。
- 本文件只做“地图与职责”，不再记录逐条开发日志。

## 1. 目录地图（先看这里）
- `crates/`：Rust 编译器主实现（稳定链路、CI 主体）。
- `lencyc/`：Lency 自举编译器（当前重点：Lexer/Parser/Sema 逐步对齐）。
- `lib/`：标准库源码（Rust/Lency 两侧都会受影响）。
- `tests/integration/`：Rust 侧集成测试。
- `tests/example/`：Lency 侧示例/实验测试。
- `scripts/run_checks.sh`：Rust 侧固定检查入口（不接收参数）。
- `scripts/run_lency_checks.sh`：Lency 侧固定检查入口（不接收参数）。
- `prompt/sprint/status.md`：当前 sprint 状态与里程碑。
- `prompt/artifacts/`：任务记录（task / plan / walkthrough）。
- `docs/`：用户文档（语言行为变化时必须同步）。

## 2. 协作与记录规则
- 进度状态：只更新 `prompt/sprint/status.md`。
- 任务过程：写入 `prompt/artifacts/` 对应文件。
- 架构变化：必要时补充到本文件“长期约定”，不要写流水账。
- 每次改动结束必须运行：
  - `./scripts/run_checks.sh`
  - `./scripts/run_lency_checks.sh`

## 3. CI 触发约定（摘要）
- CI 先按路径判定改动作用域，再触发对应 job。
- Rust 作用域：`crates/**`、`tests/integration/**`、以及共享项（如 `lib/**`、部分脚本/workflow）。
- Lency 作用域：`lencyc/**`、`tests/example/**`、以及共享项（如 `lib/**`、部分脚本/workflow）。
- `macos-check` 当前仅跟随 Rust 作用域触发（main 分支或手动触发）。

## 4. 当前工作焦点（自举）
- 已完成：Parser/AST 模块化拆分（`lencyc/syntax/{parser,ast}/...`）。
- 已支持：`break/continue` 语句及循环外非法位置约束（parser 直接报错）。
- 已支持：C 风格 `for` 语句基础解析（当前通过 parser 反糖到 `while`）。
- 当前策略：按语法特性小步增量推进，每次增量后立刻跑 Lency 检查，避免回归。
- 下一阶段：在保持可运行的前提下逐步补齐语句与语义能力。
