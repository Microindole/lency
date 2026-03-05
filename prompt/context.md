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
- 冲刺治理约束：`prompt/sprint/` 仅保留 `status.md` 作为当前真相来源；历史 `plan_*.md` / `roadmap.md` 不再维护，过期即删除。
- Lency 语法检查约定：`run_lency_checks.sh` 会优先使用 `lencyc build --check-only` 对 `lencyc/driver/test_entry.lcy` 与 `lencyc/driver/main.lcy` 做入口级语法检查；若未来该参数缺失，脚本才会回退为跳过并由完整 build 覆盖。
- 每次改动结束必须运行：
  - `./scripts/run_checks.sh`
  - `./scripts/run_lency_checks.sh`

## 3. CI 触发约定（摘要）
- CI 先按路径判定改动作用域，再触发对应 job。
- Rust 作用域：`crates/**`、`tests/integration/**`、以及共享项（如 `lib/**`、部分脚本/workflow）。
- Lency 作用域：`lencyc/**`、`tests/example/**`、以及共享项（如 `lib/**`、部分脚本/workflow）。
- `macos-check` 当前仅跟随 Rust 作用域触发（main 分支或手动触发）。
- Release 自动化：新增 `.github/workflows/release.yml`，当 push `v*` tag 时自动构建 Linux 产物并创建 GitHub Release（附 `tar.gz` 与 `sha256`）。

## 4. 当前工作焦点（自举）
- 已完成：Parser/AST 模块化拆分（`lencyc/syntax/{parser,ast}/...`）。
- 已支持：`break/continue` 语句及循环外非法位置约束（parser 直接报错）。
- 已支持：C 风格 `for` 语句基础解析（当前通过 parser 反糖到 `while`）。
- 语义修正：`for` 反糖路径下，`continue` 已确保先执行 `increment`（且不影响嵌套循环）。
- 解析边界：`for` 当前支持 `var` 或表达式初始化（如 `for var i = ...;` / `for i = ...;`）。
- 表达式能力：parser 已支持 `call` 与 `member` 链（`foo(a,b)`、`obj.method()`），并支持字符串字面量（`"text"`）。
- 数字字面量：lexer 已支持 `int/float/scientific`（如 `1`、`3.14`、`1.23e-4`、`9E+2`）。
- 字符串/字符字面量：lexer 已支持字符串转义扫描（如 `\"`、`\\n`）与字符字面量（如 `'a'`、`'\\n'`）。
- Lency 自举 TODO 状态：`lencyc/` 目录内 `TODO` 已清零；当前剩余 TODO 仅在 `lib/std` 与 Rust 编译器路径。
- 自举语义骨架：已添加最小 `name resolution`（变量定义/引用检查）并接入 `test_entry` 烟雾验证。
- 语义测试：`test_entry` 已补 resolver 负例（undefined/duplicate），不再只测正例。
- 回归结构化：测试样例已抽离到 `lencyc/driver/test_cases.lcy`，`test_entry` 改为用例编排执行。
- 最小完整链路：`lencyc/driver/main.lcy` 已串联 `Read -> Lex -> Parse -> Resolve -> Emit(AST 文本)`，默认输入 `lencyc/driver/pipeline_sample.lcy`。
- 后端演进：`lencyc` 已增加最小 LIR 文本发射（`--emit-lir`），用于对接后续 Rust LLVM backend；当前默认 emit 仍为 AST 文本以保持自举稳定。
- 回归约束：`run_lency_checks.sh` 已纳入 `tests/example/lencyc_lir_*.lcy` 用例，固定校验自举 `--emit-lir` 输出结构。
- Rust 后端进展：`crates/lency_cli` 已支持最小 `.lir -> LLVM IR -> executable` 构建路径（`lencyc build file.lir`），并接入 Lency 侧脚本的端到端冒烟。
- Rust `.lir` backend 能力增量：已支持最小外部函数调用 lowering（`call %foo(...)` -> LLVM `declare i64 @foo(...)` + `call`）。
- Rust `.lir` backend 内建符号映射：`arg_count` 已映射 `@lency_arg_count`，`arg_at` 已映射 `@lency_arg_at`（最小 backend 以 pointer-as-value 方式承载）。
- Rust `.lir` backend builtin 映射已扩展：`int_to_string` -> `@lency_int_to_string`、`file_exists/is_dir` -> `@lency_file_exists/@lency_file_is_dir`。
- Rust CLI 模块化：`crates/lency_cli/src/main.rs`、`lir_backend` 已拆分为目录模块，入口文件不再承载大段实现逻辑。
- LIR 回归样例扩展：`tests/example` 新增 `lencyc_lir_unary_logic.lcy` 与 `lencyc_lir_break_continue.lcy`，已纳入 `run_lency_checks.sh`。
- 可用性打通：新增 `scripts/lency_selfhost_build.sh`，提供 `.lcy -> self-host emit-lir -> Rust backend build` 的一键构建路径，并已接入 Lency 检查脚本闭环验证。
- 运行闭环：新增 `scripts/lency_selfhost_run.sh`，提供 `.lcy -> self-host build -> run` 一键运行路径（支持参数透传与期望退出码校验），并已接入 Lency 检查脚本。
- 运行闭环回归：`tests/example/lencyc_run_args.lcy` 已覆盖 `arg_count + arg_at`，`run_lency_checks.sh` 第 10 步不再依赖绕过用例。
- 运行时映射回归：`tests/example/lencyc_run_int_to_string.lcy` 已接入 `run_lency_checks.sh` 第 11 步，固定校验最小 runtime builtin 映射链路。
- 解析可用性修复：`lencyc` resolver 已预载最小 prelude 符号，目标源码中的 `arg_count()/arg_at()` 等内建符号不再因未声明而解析失败。
- 语义约束增量：`lencyc` resolver 已加入 builtin 调用参数个数校验（固定 arity），`test_entry` 已覆盖正/负例回归。
- 语义约束增量：`resolve_function_body` 已加入最小 return 合法性检查（value-return 函数禁止 `return` 空值，且要求可达 value-return），并已接入正/负例回归。
- 语义约束增量：`lencyc` resolver 已加入最小类型一致性检查（`int/bool/string/float`），覆盖赋值、一元、二元、逻辑表达式路径，并已接入 `test_entry` Step 16 回归。
- 声明解析增量：parser 已支持最小函数声明骨架（`int/string/bool/void/float name(...) { ... }`），并在 AST 中记录参数类型 token kind。
- 调用语义增量：resolver 已支持用户函数签名预扫描（返回类型 + 参数类型 + arity），并支持“先调用后声明”场景。
- 语义约束增量：用户函数调用已接入参数类型校验，函数 `return` 已接入返回类型校验，并已接入 `test_entry` Step 18 回归。
- 架构演进：`resolver` 已按 `resolver.lcy + resolver/core.lcy + resolver/expr.lcy` 拆分，规避单文件超 500 行限制。
- 兼容性约束：当前 self-host runtime builtin 仍有 pointer-as-value 历史语义，`arg_at/int_to_string/float_to_string/bool_to_string` 在 resolver 中暂按 `unknown` 返回类型处理，避免误杀现有运行闭环用例。
- 文档治理增量：`docs/` 已清理过时实现状态与坏链接（补齐 `types/primitives.md`、`stdlib/hashmap.md`，同步脚本文档到当前检查链路）。
- 当前策略：按语法特性小步增量推进，每次增量后立刻跑 Lency 检查，避免回归。
- 下一阶段：在保持可运行的前提下逐步补齐语句与语义能力。
