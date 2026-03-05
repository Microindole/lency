# Sprint 18 Implementation Plan (Semantic Analysis)

## 目标
以“最小语义闭环 + 立即回归”为节奏推进 `lencyc` 自举语义层，优先阻断明显非法程序。

## 本轮已完成 (2026-03-04)
1. 在 `lencyc/sema/resolver.lcy` 增加 builtin 函数签名表（参数个数）。
2. 在 `resolve_expr(EXPR_CALL)` 增加 builtin 调用参数个数校验（arity mismatch 报错）。
3. 在 `lencyc/driver/test_cases.lcy` 新增 `src_resolver_builtin_arity_ok` 与 `src_resolver_builtin_arity_bad`。
4. 在 `lencyc/driver/test_entry.lcy` 新增 Step 15，覆盖 builtin arity 正/负例回归。
5. 在 `lencyc/sema/resolver.lcy` 增加函数体 return 约束：
   - value-return 函数禁止 `return` 空值。
   - 函数体必须保证存在可达 value-return（支持 block/if-else 双分支返回判定）。
6. 在 `lencyc/driver/test_cases.lcy` 与 `lencyc/driver/test_entry.lcy` 增加函数体 return 约束正/负例回归。

## 本轮已完成补充 (2026-03-05)
1. 在 `lencyc/sema/resolver.lcy` 增加最小类型跟踪与一致性检查（`int/bool/string/float`）。
2. 在赋值、一元、二元、逻辑表达式分支接入最小类型约束与诊断。
3. 在 `lencyc/driver/test_cases.lcy` 新增类型一致性正/负例用例。
4. 在 `lencyc/driver/test_entry.lcy` 新增 Step 16，接入类型一致性回归。
5. 对 `arg_at/int_to_string/float_to_string/bool_to_string` 采用 `unknown` 返回类型兜底，兼容当前 self-host runtime 回归用例（pointer-as-value 语义）。
6. 在 `lencyc/syntax/parser/decl.lcy` 接入最小函数声明骨架解析（`int/string/bool/void/float` 起始）。
7. 在 `lencyc/sema/resolver.lcy` 增加用户函数 arity 预扫描，并在 `EXPR_CALL` 复用统一 arity 校验逻辑。
8. 在 `lencyc/driver/test_cases.lcy` 与 `lencyc/driver/test_entry.lcy` 增加 Step 17 用户函数 arity 正/负例回归。
9. 在 token/lexer 层新增 `T_FLOAT`，补齐 `float` 关键字在函数签名中的词法支持。
10. 在 AST/Parser 层新增函数参数类型记录（`param_kinds`），让 resolver 可读取签名类型。
11. 在 resolver 层新增用户函数签名表（返回类型 + 参数类型），并接入调用参数类型校验与 return 返回类型校验。
12. 在 `lencyc/sema/resolver` 目录完成模块拆分（`resolver.lcy`/`core.lcy`/`expr.lcy`），规避单文件超限。
13. 在 `test_cases/test_entry` 新增 Step 18：用户函数类型签名正例、参数类型负例、返回类型负例、float 签名正例。

## 下一步 (按优先级)
1. 调用语义扩展
   - 自定义类型（`T_IDENTIFIER`）参数/返回签名接入与类型级调用校验。

## 质量门禁
每次改动结束必须执行：
1. `./scripts/run_checks.sh`
2. `./scripts/run_lency_checks.sh`

## 当前技术债
1. 语义诊断仍是 `print` 文本，未统一 Reporter。
2. 函数签名当前仅支持基础内建类型 token，自定义类型名尚未接入签名语义。
