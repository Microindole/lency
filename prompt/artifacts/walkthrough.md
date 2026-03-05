# Sprint 18 Walkthrough (2026-03-05)

## 当前进度
1. 语义层已具备最小 name resolution 与函数体作用域入口。
2. 本轮新增 builtin 调用参数个数校验，解析后可直接阻断明显非法调用。
3. 本轮新增函数体 return 约束（禁止 void-return、要求可达 value-return）。
4. 自举回归已覆盖 builtin arity 与 function-return 正/负路径。
5. 本轮补齐最小类型一致性检查（`int/bool/string/float`）并接入回归。
6. 本轮接入最小函数声明骨架解析 + 用户函数 arity 校验（含先调用后声明）。
7. 本轮补齐用户函数类型签名校验（参数类型 + 返回类型）并接入 Step 18 回归。

## 本轮改动明细
1. `lencyc/sema/resolver.lcy`
   - 新增 `BuiltinArity` 与 `builtin_arities`。
   - 新增 `preload_builtin_arities()` 与 `lookup_builtin_arity()`。
   - 在 `EXPR_CALL` 分支新增参数个数检查：`expected != actual` 时报错。
   - 在函数体解析路径新增 return 约束检查：`return` 形态合法性 + 可达 value-return 判定。
2. `lencyc/driver/test_cases.lcy`
   - 新增 `src_resolver_builtin_arity_ok()`。
   - 新增 `src_resolver_builtin_arity_bad()`。
   - 新增 `src_resolver_function_body_missing_return()`。
   - 新增 `src_resolver_function_body_void_return_bad()`。
   - 新增 `src_resolver_function_body_if_else_return_ok()`。
3. `lencyc/driver/test_entry.lcy`
   - 新增 Step 15 builtin arity 回归，接入统一通过/失败断言。
   - 新增函数体语义断言 helper，并接入 function-return 正/负例。
   - 新增 Step 16 类型一致性回归（正例 + 负例）。
   - 新增 Step 17 用户函数 arity 回归（正例 + 负例）。
   - 新增 Step 18 用户函数类型签名回归（参数类型负例、返回类型负例、float 正例）。
4. builtin 返回类型兼容策略
   - `arg_at/int_to_string/float_to_string/bool_to_string` 暂按 `unknown` 处理，兼容当前 runtime pointer-as-value 用例。
5. `lencyc/syntax/parser/decl.lcy`
   - 新增最小函数声明骨架解析：`int/string/bool/void/float name(type p, ...) { ... }`。
   - 参数类型 token 写入 `Stmt.param_kinds`，供 resolver 做签名类型校验。
6. `lencyc/sema/resolver/*`
   - `resolver.lcy` 仅保留结构与入口，`core.lcy`/`expr.lcy` 承载实现，避免单文件超限。
   - 用户函数签名表新增返回类型与参数类型；`EXPR_CALL` 新增参数类型校验；`STMT_RETURN` 新增返回类型校验。

## 验证方式
1. 运行 `./scripts/run_checks.sh`。
2. 运行 `./scripts/run_lency_checks.sh`。

## 未尽事宜
1. 自定义类型（`T_IDENTIFIER`）签名尚未进入语义类型系统，当前只覆盖内建基础类型 token。
