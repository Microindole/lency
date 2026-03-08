# Lency 项目上下文入口

## 0. 最高准则
- 语言与设计哲学：`assets/Lency.txt`、`assets/design_spec.md`（冲突时以这两个文件为准）。
- `xtask` 是规范主入口：`cargo run -p xtask -- check-rust`、`cargo run -p xtask -- check-lency`。
- `prompt/sprint/status.md` 是唯一状态真相来源，本文件只保留长期协作上下文与基线。
- Phase 0 能力矩阵真表：`prompt/artifacts/capability_matrix.md`。

## 1. 当前基线（2026-03-08）
- Rust 主编译器（`crates/`）源码文件数：175。
- Lency 自举编译器（`lencyc/`）源码文件数：27。
- Rust 集成测试文件数（`tests/integration/`）：74。
- Lency 示例测试文件数（`tests/example/`）：10。
- 当前判断：`lencyc` 已打通最小闭环，但能力远未接近 Rust 主链路；此前“~98% 准备度”判定失真，已废弃。

## 2. 能力对照（Rust vs Lency 自举）
- 词法/语法：
  - Rust：已覆盖 `const/import/extern/trait/enum/match/null/?. /??/vec/Ok/Err` 等。
  - Lency：当前聚焦 `var/if/while/for/struct/impl/return` 与基础字面量/运算。
  - TODO: 补齐 `match/null` 与相关语法节点（`enum` 当前为 unit variant 子集）。
- 语义：
  - Rust：Resolver + TypeInfer + TypeCheck + NullSafety 分层较完整。
  - Lency：当前为最小语义约束（name resolution、基础类型一致性、函数签名与 arity、impl/struct 最小校验）。
  - TODO: 扩展 nullable/result/enum+match 语义与更完整控制流返回分析。
  - TODO: `import` 当前仅做 alias 最小绑定，尚未做模块加载与符号导入规则。
- 后端：
  - Rust：AST -> LLVM IR -> 可执行链路成熟。
  - Lency：`AST/LIR` 最小发射 + Rust `.lir` backend 冒烟，仍有子集约束。
  - FIXME: `crates/lency_cli/src/lir_backend/compile.rs` 仍存在 builtin 子集、call/member lowering 限制。
- 工具链：
  - Rust：`compile/run/check/build/repl` 路径稳定。
  - Lency：`lencyc` + `xtask check-lency` 闭环可用，但语言特性覆盖不足。

## 3. 已落地的自举关键增量
- `TypeRef` 已接入函数签名（`return_type + param_types`），替代旧并行字段。
- `struct` 声明已从空骨架升级为字段解析（`Type field` 列表）。
- resolver 已接入 `struct` 字段重复与未知类型校验。
- `const` 声明已接入 lexer/parser/AST/resolver，禁止对 `const` 变量赋值。
- `import/extern` 声明已接入 lexer/parser/AST/resolver，`extern` 可参与调用 arity 校验。
- `enum` 声明已接入 lexer/parser/AST/resolver（当前仅 unit variant，payload/match 待后续）。
- 回归已扩展到 Step 28，并通过 `check-lency` 全链路验证。
- AST 构造器已切换为 `make_stmt_base + 局部覆写` 工厂模式，新增 `Stmt` 字段时只需集中修改基座，显著降低连锁改动面。
- parser 声明路径已抽出 `parse_signature_param_list()` 公共 helper，减少 `function/extern` 参数解析重复逻辑。
- 已引入 `Program(decls + statements)` 过渡模型与 `parse_program()/resolve_program()` 入口，为后续 Decl/Stmt 解耦与 payload 化迁移提供兼容路径。
- resolver 预加载已从 `Decl` 视图直连（不再依赖 `Decl -> Stmt` 中转），迁移方向保持单向解耦。
- `test_entry` 与拆分的 `test_steps_*` 已切换到 `parse_program()/resolve_program()`，过渡入口已被回归覆盖。
- `resolve_statements()` 兼容入口已移除，resolver 统一从 `resolve_program()` 进入。
- 拆分的声明测试步骤（`const/enum/import-extern/signature/struct`）已统一使用 `parse_program()` 读取 AST，`parse()` 仅保留在 `test_entry` 的词法/语法恢复类场景。
- `test_entry` 的 resolver smoke 路径已去除手工 `extract_declarations + make_program` 组装，改为直接复用 `parse_program()` 结果。
- `test_entry` 的声明骨架断言步骤（Step 20/21/22）已切换到 `parse_program()`，减少声明测试路径对旧 `parse()` 的依赖面。
- `test_entry` 的 `expect_parse_error` 与 function-body resolver helper 已统一走 `parse_program()`，不再依赖旧 `parse()` 入口。
- `lencyc/driver` 已完成 `parse()` 调用清零，统一通过 `parse_program()` 获取 `Program`（再按需读取 `program.statements`）。
- parser 入口已收敛为 `parse_program()` 单一接口，旧 `parse()` 兼容方法已移除。
- resolver 已清理旧 `Stmt` 入口遗留：删除 `preload_user_type_declarations/preload_user_function_signatures` 死代码，仅保留 Decl/Program 预加载路径。
- resolver 预加载阶段已对 `DECL_UNKNOWN` 给出显式错误，避免静默跳过导致问题被吞掉。
- `test_entry` 已新增 `DECL_UNKNOWN` 预加载防御回归，确保该错误路径不会被回归吞掉。
- 自举测试支持层已新增 `driver/test_support.lcy`，并先迁移 `test_steps_const` 的 resolve 断言 helper，开始收敛重复逻辑。
- 共享测试 helper 已继续迁移到 `test_steps_enum`、`test_steps_struct`、`test_steps_import_extern`，重复 resolve 断言代码进一步减少。
- `test_entry` 主流程中的 resolve 正/负例断言也已切换到 `test_support` 共享 helper，driver 测试断言逻辑已基本统一。
- `function-body` resolve 正/负例断言也已迁入 `test_support`，`test_entry` 继续瘦身。
- `parse-error` 断言已统一迁入 `test_support`（`parser_frontend/import_extern` 本地重复 helper 已移除）。
- syntax 声明过渡层已删除未使用的 `extract_non_declaration_statements`，减少无效 API 面。
- `test_entry` 已进一步拆分：Step 3-10（解析前端回归）迁移到 `test_steps_parser_frontend.lcy`，主入口体积继续下降。
- resolver 结构继续拆分：`resolve_stmt` 已迁移到 `sema/resolver/stmt.lcy`，降低 `core.lcy` 单文件复杂度。
- resolver return-flow 分析也已拆分到 `sema/resolver/return_flow.lcy`，继续压低 `core.lcy` 体积与职责耦合。
- resolver 声明语句分支已拆分到 `sema/resolver/decl_stmt.lcy`（`resolve_decl_stmt`），`resolve_stmt` 仅保留分派与普通语句处理。
- resolver 声明处理内部已切换到 `Stmt -> Decl` 视图再执行（`resolve_decl`），减少对 `Stmt` 声明字段的直接耦合。
- `resolve_import/extern/enum` 已补齐 `Decl` 入口（`resolve_*_decl`），声明语义路径不再需要 `Decl -> Stmt` 回转桥接。
- parser `parse_program()` 已改为单趟构建 `statements + decls`，移除声明二次提取流程，继续收敛桥接层复杂度。
- AST `Stmt` 已完成声明 payload 化：声明细节统一下沉到 `stmt.decl`（`Decl`），移除 `Stmt` 上的 `return_type/params/param_types/struct_fields` 散落字段，新增声明节点不再触发 `Stmt` 结构体字段扩散修改。
- AST 声明构造链已统一为 `make_decl_*` + `make_stmt_*`，`stmt_to_decl` 改为直接读取 payload（带 kind 一致性防御）。
- parser/syntax/driver 侧声明断言与打印已统一走 `Decl` 视图（`stmt_to_decl`），减少对 `Stmt` 内部布局的耦合。
- 收尾说明：`status.md` 已移除两条过时 `FIXME`（“AST 未 payload 化 / resolver 未分层”），并替换为当前真实剩余风险条目。
- `xtask check-rust` 已纳入 `.lcy` 集成用例（`tests/integration/*.lcy`）执行，不再只依赖 Rust 单元/文档测试覆盖。
- `run_lcy_tests` 脚本已修正项目根路径解析（`scripts/* -> repo root`），消除此前误报“未找到 tests/integration”导致的假通过。

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
- 进度只在 `prompt/sprint/status.md` 更新，不在多处维护冲突状态。
- 每次改动后必须执行：
  - `cargo run -p xtask -- check-lency`
  - `cargo run -p xtask -- check-rust`
- TODO: 任何新增未完成设计项必须在对应模块或文档明确 `TODO`，禁止口头挂账。
- FIXME: 任何已知错误路径必须明确 `FIXME` 与收敛计划，禁止“先过再说”。
