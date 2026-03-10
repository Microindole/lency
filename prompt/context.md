# Lency 项目上下文入口

## 0. 最高准则
- 语言与设计哲学：`assets/Lency.txt`、`assets/design_spec.md`（冲突时以这两个文件为准）。
- 协作规则补充：`prompt/rules.md`（与本文件并行生效，用于约束日常实现与重构风格）。
- `xtask` 是规范主入口：`cargo run -p xtask -- auto-check`（按改动范围自动执行 `check-rust`/`check-lency`）。
- `prompt/context.md` 是日常实时同步入口（每轮实现后都要更新）；`prompt/sprint/status.md` 仅做阶段里程碑汇总与发布级复盘。
- Phase 0 能力矩阵真表：`prompt/artifacts/capability_matrix.md`。

## 0.5 Agent 快速执行入口（必读，避免反复打转）
- 当前真实阶段：Sprint 18（Semantic Analysis）进行中，主目标不是继续雕 parser 细节，而是提升语义拦截密度并推进自举可运行子集。
- 自举链路现状：
  - 已打通：`Rust(v0) -> Lency(stage1) -> Lency(stage2/stage3) -> bootstrap-check(stage2/stage3 LIR 收敛)`
  - 已有 CI：`.github/workflows/bootstrap.yml`（仅手动或 `bootstrap-check/**` tag 触发）
  - 现状判定：具备“最小可用自举闭环”，但未达到“接近 Rust 主链路使用水平”。
- 下一步唯一优先级（按顺序，不要跳）：
  1. `match` 复杂模式语义继续补齐：优先 guard 组合/嵌套模式边界，新增 resolver 正负例。
  2. enum 类型流继续补齐：复杂控制流与多层调用组合，新增可复现负例，禁止静默放行。
  3. 后端 `match` lowering 扩展：在现有 `number/bool/null/char + _ + 基础 guard` 之上继续扩展，并新增 runtime 回归。
- 每次改动的硬验收（必须同时满足）：
  - 新增或更新对应 `tests/example/selfhost/driver/steps/*` 的语义回归（不是只改 runtime）。
  - 新增或更新对应 `tests/example/runtime/*` 的端到端回归（涉及 lowering 时）。
  - 执行 `cargo run -p xtask -- auto-check` 全绿。
  - 同步更新本文件与必要的 `docs/*`（语言行为变更必须落文档）。
- 禁止事项（当前阶段）：
  - 禁止为了“看起来推进”而只做目录重排/命名重构。
  - 禁止只改 parser/syntax 而不补 sema 或 runtime 对应回归。
  - 禁止引入新的 `TYPE_UNKNOWN` 逃逸路径来掩盖真实错误。

## 1. 当前基线（2026-03-10）
- Rust 主编译器（`crates/`）文件数：178。
- Lency 自举编译器（`lencyc/`）`.lcy` 文件数：34。
- Rust 集成测试文件数（`tests/integration/`）：75。
- Lency 示例测试文件数（`tests/example/`）：38（已按 `lir/runtime/parser/modules/selfhost` 分层）。
- 统计口径：递归文件计数（Windows `Get-ChildItem -Recurse -File`；`lencyc` 按 `*.lcy` 计）。
- 当前判断：`lencyc` 已打通最小闭环，但能力远未接近 Rust 主链路；此前“~98% 准备度”判定失真，已废弃。

## 1.5 版本定义（回答版本问题时以此为准）
- Rust 工具链版本：`rustc 1.87.0 (17067e9ac 2025-05-09)`。
- Rust 主链路 crate 版本：当前工作区 `crates/*/Cargo.toml` 统一为 `0.1.0`。
- LLVM 绑定口径：workspace `inkwell` feature 为 `llvm15-0`（见根 `Cargo.toml`）。
- Lency 自举编译器版本：`lencyc/driver/main.lcy` banner 为 `Lency Self-hosted Compiler (v0.1.0)`。

## 2. 能力对照（Rust vs Lency 自举）
- 一句话：Lency 已具备最小自举闭环，但与 Rust 主链路仍有代差，当前应优先补语义密度与 lowering 覆盖，而不是继续打磨 parser 外形。
- 前端：`match/enum payload/import/extern/null/泛型入口` 已接入；复杂模式与更高阶 guard 仍有缺口。
- 语义：已覆盖 name resolution、基础 type check、函数签名、enum/match 第一版、`std.*` 自动签名导入；复杂控制流类型流仍需增强。
- 后端：selfhost LIR 已覆盖 `match(number/bool/null/char + _ + 基础 guard)` 子集；复杂 pattern lowering 仍是待办。
- 工具链：`auto-check` 与 `bootstrap-check` 已落地，手动/tag 触发的重型收敛验证已具备。

## 3. 已落地的自举关键增量
- 前端与结构收敛：
  - parser 入口统一为 `parse_program()`，resolver 入口统一为 `resolve_program()`。
  - AST 声明 payload 化（`stmt.decl`）已完成，resolver 已改为 Decl 视图直连，减少 `Stmt` 声明字段扩散。
  - `tests/example/selfhost/driver` 已按 `steps/*` 分层，且 `test_support` 共用断言已接入主路径。
- 语义能力基线（可用）：
  - `const/import/extern/enum/match/null` 语义第一版可用。
  - `enum payload` 构造与匹配、`match guard`（`if (cond)` 且 `bool`）已接入。
  - 非 enum `match` literal pattern 已覆盖 `number/string/bool/null/char` 与 `_`，并有 resolver 正负例。
  - enum 类型流已覆盖函数返回、match 中间表达式、赋值链、分组 callee 调用链等主路径。
- 后端与运行时基线（可跑）：
  - selfhost `match lowering` 当前覆盖 `number/bool/null/char + _ + 基础 guard` 子集。
  - runtime 回归已覆盖 `match_guard`、`match_bool_null`、`match_char`。
  - `match` 更复杂 pattern lowering 仍是后续增量（见 TODO）。
- 工具链与收敛验证：
  - `cargo run -p xtask -- auto-check` 为唯一日常主入口（支持 docs-only 快速模式）。
  - `cargo run -p xtask -- bootstrap-check` 已落地；`bootstrap.yml` 仅手动或 `bootstrap-check/**` tag 触发。
- 追溯与细节：
  - 详细历史与迁移细节在 `prompt/sprint/status.md` 与 `prompt/artifacts/*`，本文件保留“可执行摘要 + 当前边界”。

## 4. 目录与职责
- `crates/`：Rust 主编译器与主工具链。
- `lencyc/`：自举编译器实现（当前主战场）。
- `lib/`：标准库源码（Rust/Lency 双侧受影响）。
- `tests/integration/`：Rust 侧集成测试。
- `tests/example/`：Lency 自举侧示例与回归输入。
- `docs/`：对外文档，语言行为变化必须同步。
- `prompt/sprint/status.md`：里程碑与阶段状态。
- `prompt/artifacts/`：任务拆解与实现记录。

## 5. 协作与验收规则
- 日常进度与实现细节实时更新 `prompt/context.md`；`prompt/sprint/status.md` 仅在阶段切换/里程碑收口时更新，避免频繁漂移与重复维护。
- 每次改动后必须执行：
  - `cargo run -p xtask -- auto-check`
- TODO: 任何新增未完成设计项必须在对应模块或文档明确 `TODO`，禁止口头挂账。
- FIXME: 任何已知错误路径必须明确 `FIXME` 与收敛计划，禁止“先过再说”。
