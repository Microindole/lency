# Sprint 状态总结

更新时间：2026-03-08

## 0. 当前结论（先看）
- `lencyc` 已完成最小自举闭环：`Read -> Lex -> Parse -> Resolve -> Emit(AST/LIR)`。
- 与 Rust 主链路仍有显著差距，不能再使用“~98% 准备度”这类失真数字。
- 当前主线：先收尾 Parser/Decl 可用性，再补 Sema 深度，最后扩后端能力。

## 1. 双链路现状基线
- Rust 主链路：
  - `crates/` 源码文件：175
  - `tests/integration/` 文件：74
  - 能力层级：语法/语义/单态化/LLVM codegen/CLI 已成体系
- Lency 自举链路：
  - `lencyc/` 源码文件：25
  - `tests/example/` 文件：23（已按 `lir/runtime/parser/modules/selfhost` 分层）
  - 能力层级：最小语法与最小语义可运行，后端与类型系统仍是子集

## 2. Sprint 状态

### Sprint 17：自举 Parser（收尾中）
目标：声明解析从“最小骨架”推进到“可用级”。

已完成：
- [x] Parser/AST 模块化拆分
- [x] 表达式优先级链、`call/member`、`string/char/int/float/scientific`
- [x] 语句解析：`var/if/while/for/block/return/break/continue/expr`
- [x] `for` 反糖与 `continue` 语义修复
- [x] `TypeRef` 落地到函数签名
- [x] `struct` 字段声明解析与 AST 挂载（`struct_fields`）
- [x] parser declaration 级错误恢复同步点
- [x] `const` 声明语法接入（lexer/parser/AST）与 for-init 支持
- [x] `import/extern` 声明语法接入（含 AST/打印与同步点）
- [x] `enum` 声明语法接入（含 payload variant 语法 + parser 回归）
- [x] `match` 最小可用语法接入（lexer/parser/AST/resolver smoke）
- [x] AST `Stmt` 构造器工厂化（`make_stmt_base`），降低节点扩展时的全局修改面
- [x] Parser 声明参数解析公共化（`parse_signature_param_list`）
- [x] 过渡入口打通：`parse_program()` + `resolve_program()`（保留 `parse()/resolve_statements()` 兼容）
- [x] Resolver 预加载改为直接消费 `Decl` 视图（去除 `Decl -> Stmt` 预加载中转）
- [x] 测试入口迁移：`test_entry` 与 `test_steps_*` 统一走 `parse_program()/resolve_program()`
- [x] 移除 `resolve_statements()` 兼容 API，resolver 统一入口为 `resolve_program()`
- [x] 拆分声明测试步骤统一使用 `parse_program()`；`parse()` 仅保留在 `test_entry` 词法/恢复类测试
- [x] `test_entry` resolver smoke 测试改为直接消费 `parse_program()`，移除手工 Program 拼装桥接
- [x] `test_entry` Step20/21/22（struct/impl 声明骨架）已切换为 `parse_program()` 断言
- [x] `test_entry` 的 parse-error/function-body helper 统一为 `parse_program()` 输入模型
- [x] `lencyc/driver` 目录内 `parse()` 调用清零，统一通过 `parse_program()` 获取 Program
- [x] parser 入口收敛：仅保留 `parse_program()`，移除 `parse()` 兼容方法
- [x] resolver 删除旧 `Stmt` 预加载死代码（`preload_user_type_declarations/preload_user_function_signatures`）
- [x] resolver 预加载对 `DECL_UNKNOWN` 改为显式报错，移除静默跳过策略
- [x] `DECL_UNKNOWN` 预加载防御回归已接入 `test_entry`（防止静默吞错回归）
- [x] 删除 syntax Decl 过渡层未使用 API：`extract_non_declaration_statements`
- [x] `test_entry` Step3-10 拆分到 `test_steps_parser_frontend`，继续收敛入口文件体积
- [x] resolver `resolve_stmt` 拆分到 `sema/resolver/stmt.lcy`，继续收敛单文件复杂度
- [x] resolver return-flow 分析拆分到 `sema/resolver/return_flow.lcy`
- [x] resolver 声明语句分支拆分到 `sema/resolver/decl_stmt.lcy`，`resolve_stmt` 收敛为“普通语句 + 声明分派”
- [x] resolver 声明语义处理切换为 `Stmt -> Decl` 视图驱动（`resolve_decl`），降低声明字段变更的扩散面
- [x] `resolve_import/extern/enum` 已新增 `Decl` 入口，移除声明语义中的 `Decl -> Stmt` 回转桥接
- [x] parser `parse_program()` 改为单趟累积 `decls`，移除 `extract_declarations` 的二次扫描路径
- [x] AST `Stmt` 声明字段完成 payload 化：声明数据下沉到 `stmt.decl`（`Decl`），消除 `Stmt` 结构体的声明字段扩散点
- [x] 声明构造器统一为 `make_decl_*`，`stmt_to_decl` 改为 payload 直读并增加 kind 一致性防御
- [x] parser/driver/printer 的声明断言与输出统一走 `Decl` 视图，降低 AST 布局变更影响面
- [x] `xtask check-rust` 已纳入 `tests/integration/*.lcy` 门禁（通过 `run_lcy_tests`），补齐 Rust 主链路对 `.lcy` 集成用例的基础覆盖
- [x] Linux/Windows `run_lcy_tests` 已修正项目根目录解析，消除“目录不存在导致假通过”的脚本缺陷
- [x] 新增 `driver/test_support.lcy`，`test_steps_const` 已切换到共享 resolve 断言 helper
- [x] 共享 resolve 断言 helper 已扩展到 `test_steps_enum/test_steps_struct/test_steps_import_extern`
- [x] `test_entry` resolve 正/负例断言已切换到 `test_support` 共享 helper
- [x] `function-body` resolve 正/负例断言已切换到 `test_support` 共享 helper
- [x] `parse-error` 断言已切换到 `test_support` 共享 helper（`parser_frontend/import_extern`）

未完成：
- [ ] TODO: 声明层与表达式层的泛型语法入口统一

### Sprint 18：自举 Semantic Analysis（进行中）
目标：从“最小约束”扩展到“可拦截主要错误”的语义层。

已完成：
- [x] 最小 name resolution（undefined/duplicate/out-of-scope/shadowing）
- [x] builtin arity 校验
- [x] 函数 `return` 最小约束
- [x] 基础类型一致性（`int/bool/string/float`）
- [x] 用户函数签名/调用校验（含先调用后声明）
- [x] `impl` 目标类型存在与方法重名校验
- [x] `struct` 字段重名与未知类型校验
- [x] `const` 赋值拦截（assignment to const）与遮蔽场景回归
- [x] `extern` 签名预加载与调用 arity 校验
- [x] `import` 最小 alias 绑定骨架
- [x] `enum` duplicate variant 最小语义校验

未完成：
- [ ] TODO: nullable/result 语义规则落地（不是 `unknown` 兜底）
- [ ] TODO: `enum + match` 语义一致性与穷尽性检查
- [ ] TODO: import 模块加载与符号导入规则（当前仅 alias 绑定）
- [ ] TODO: `enum` payload 类型与构造/匹配语义（语法已接入，语义与穷尽性未完成）

## 3. 与 Rust 使用水平的差距评估（2026-03-07）
- 前端语法能力：约 35%
- 语义能力：约 30%
- 后端能力：约 20%
- 工具链可用性：约 40%
- 综合：约 30%（距离“接近 Rust 主链路使用水平”约差 70%）

## 4. 分阶段执行计划（完整）

### Phase 0：基线冻结（1 天）
- 产出：
  - 能力矩阵（语法/语义/后端/stdlib/CLI）落地到 `prompt/artifacts/`
  - 每项状态统一为 `NotStarted/InProgress/Done`
- 验收：
  - [x] 矩阵文件提交
  - [x] `auto-check`（按改动范围）通过；涉及双侧变更时需 `check-lency` + `check-rust` 均通过

### Phase 1：语法补齐第一批（2~3 周）
- 范围：`const/import/extern/enum/match/null` 与 AST 对齐
- 验收：
  - [ ] TODO: 每项均有 parser 正/负例
  - [ ] TODO: `test_entry` 分步回归持续可维护（单文件不超 500 行）

### Phase 2：语义补齐第一批（3~4 周）
- 范围：nullable/result、enum/match、import/extern 绑定
- 验收：
  - [ ] TODO: 语义负例新增 >= 40 条
  - [ ] TODO: 主要错误在 resolver 阶段失败，不落入后端

### Phase 3：泛型与 trait 最小可用（4~5 周）
- 范围：泛型参数、泛型实例化、trait/impl 匹配
- 验收：
  - [ ] TODO: `Vec<T>/Result<T,E>/trait method` 自举样例可跑通
  - [ ] TODO: 去除关键路径 `unknown` 逃逸

### Phase 4：后端能力提升（3~4 周）
- 范围：LIR 指令补齐、call/member lowering、runtime builtin 类型对齐
- 验收：
  - [ ] TODO: 关键后端 FIXME 收敛或降级为非阻塞项
  - [ ] TODO: `.lcy -> selfhost -> .lir/.exe` 回归稳定

### Phase 5：可用性冲刺（2~3 周）
- 范围：stdlib 常用能力、诊断质量、文档对齐
- 验收：
  - [ ] TODO: 15~20 个真实小程序样例通过率 > 90%
  - [ ] TODO: `docs/` 与实现一致性检查通过

## 5. 本周执行项（立即）
1. [x] Phase 0 基线矩阵文件已入库（`prompt/artifacts/capability_matrix.md`）。
2. [x] Phase 1 子项 1：`const`（语法 + 回归 + sema 最小约束）已完成（Step 26）。
3. [x] Phase 1 子项 2：`import/extern`（语法 + parser 回归 + resolver 绑定骨架）已完成（Step 27）。
4. [ ] TODO: 每个子项都配正/负例，并执行双检查。
5. [x] Phase 1 子项 3（部分）：`enum` 最小可用语法与 AST 接入已完成（Step 28）。
6. [x] Phase 1 子项 3：`match` + `enum payload` 语法与回归已落地（Step 29）。

## 6. 已知风险
- FIXME: 文档与实现存在历史错位，若不强制每次同步会继续漂移。
- FIXME: 自举 resolver 中仍有 `TYPE_UNKNOWN` 兼容路径，会掩盖真实类型错误。
- FIXME: Rust `.lir` backend 仍有 call/member lowering 子集限制。
- FIXME: 顶层声明已完成 payload 化与分层，但块内声明仍通过 `Stmt` 路径执行，后续需要评估“声明语句统一中间表示”以进一步收敛双轨分支。
- FIXME: `stmt_to_decl` 当前对声明 kind 与 payload kind 不一致仍以 `DECL_UNKNOWN` 兜底，后续需补齐诊断上下文并评估 parser 阶段前置拦截。

## 7. 本次重构收尾结论（2026-03-08）
- [x] 设计模式核心问题已收敛：声明数据不再扩散在 `Stmt` 字段中，改为 `stmt.decl` payload。
- [x] resolver 已完成“普通语句处理 vs 声明语义处理”职责分层，声明路径统一消费 `Decl` 视图。
- [x] parser 已去除声明二次扫描，`parse_program()` 单趟构建 `Program(decls + statements)`。
- [ ] TODO: `match`/`enum payload` 语义（穷尽性、绑定与类型一致性）尚未接入，属于后续主线阻塞项。
