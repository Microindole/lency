# Lency 项目上下文入口

## 0. 入口定位
- `prompt/context.md` 是唯一执行入口：只保留当前决策必需信息，不重复展开长期哲学、执行细则或阶段流水账。
- 长期设计原则看：`assets/Lency.txt`、`assets/design_spec.md`。
- 执行级协作规则看：`prompt/rules.md`。
- 阶段里程碑与历史复盘看：`prompt/sprint/status.md`。
- 能力矩阵真表看：`prompt/artifacts/capability_matrix.md`。
- 规范检查入口始终是：`cargo run -p xtask -- auto-check`。

## 1. 当前主线（2026-03-10）
- 当前阶段：Sprint 18（Semantic Analysis）。
- 主目标：继续提升语义拦截密度并维持自举主链路稳定，不要回头把 parser 当主战场。
- 当前优先级：
  1. enum 类型流继续补齐，向更复杂控制流/多层组合扩展，禁止静默放行。
  2. resolver expr visitor 化继续收口，后续增量保持按 kind 分层，不退回单函数堆叠。
  3. parser/trait 旧债按收益排序清理，不抢主线。

## 2. 当前边界
- 自举链路已打通：`Rust(v0) -> Lency(stage1) -> Lency(stage2/stage3) -> bootstrap-check(stage2/stage3 LIR 收敛)`。
- 当前判断：已具备最小可用自举闭环，但离 Rust 主链路使用水平仍有明显代差。
- 前端现状：`match/enum payload/import/extern/null/泛型入口` 已接入；parser 当前不是主瓶颈。
- 语义现状：
  - 已覆盖 name resolution、基础 type check、enum/match、guard 组合边界。
  - enum 类型流已覆盖函数返回、match 中间表达式、赋值链、grouped callee/constructor、参数透传、派生局部变量、block 遮蔽、`if/while` 写回。
  - resolver expr 已切到 visitor 风格分派骨架（`visit_expr_*`）。
  - 剩余缺口仍是“更复杂控制流/组合下的 enum 类型流增强”。
- 后端现状：
  - selfhost `match` lowering 已覆盖 `number/string/bool/null/char + _ + guard`，并支持递归 enum payload mixed pattern lowering。
  - selfhost enum 构造已收口为 `lency_enum_new0 + lency_enum_push`。
  - 2026-03-11：已补齐 selfhost string `!=` -> `cmp_str_ne` lowering，且 enum 多 payload 构造会持续转发 `lency_enum_push` 返回句柄，修复 `match_enum_payload` 在自举 runtime 链路上的崩溃风险。
  - 2026-03-11：`check-lency` 的 Step 10 已输出 runtime case / selfhost LIR / 生成可执行路径，避免 Linux CI 再次只剩 `exit code -1` 的垃圾日志。
  - 2026-03-11：runtime `lency_string_eq` 新增低地址指针防护，拦截 selfhost Linux 链路中“把标量 payload 当字符串句柄”时的直接崩溃；当前仍保留 FIXME，说明这是止血而非根因级审计完结。
  - runtime 回归已覆盖 `match_guard`、`match_guard_combo`、`match_bool_null`、`match_char`、`match_string`、`match_enum_payload`。

## 3. 当前硬约束
- 每次改动必须同时满足：
  - 有对应回归；语义改动优先补 `tests/example/selfhost/driver/steps/*`。
  - 涉及 lowering / runtime 行为时，补 `tests/example/runtime/*` 端到端回归。
  - 执行 `cargo run -p xtask -- auto-check` 并全绿。
  - 语言行为变化要同步更新本文件与必要的 `docs/*`。
- 当前禁止事项：
  - 禁止为了“看起来推进”而只做目录重排或命名重构。
  - 禁止只改 parser/syntax 而不补 sema 或 runtime 对应回归。
  - 禁止引入新的 `TYPE_UNKNOWN` 逃逸路径来掩盖真实错误。

## 4. 当前已知旧债
- TODO: enum 类型流在更复杂控制流/多层调用组合场景继续增强。
- TODO: parser trait 旧债仍在 `lencyc/syntax/parser/decl.lcy`。
- FIXME: parser 保守 lookahead 仍在 `lencyc/syntax/parser.lcy`。

## 5. 回答口径
- 版本口径：
  - Rust toolchain：`rustc 1.87.0 (17067e9ac 2025-05-09)`
  - Rust crate：workspace 当前统一 `0.1.0`
  - LLVM 口径：`inkwell` feature 为 `llvm15-0`
  - Lency selfhost：`lencyc/driver/main.lcy` banner 为 `v0.1.0`
- 若需要阶段细节、已完成项明细或历史背景，跳转到 `prompt/sprint/status.md`，不要在本文件继续堆摘要。
