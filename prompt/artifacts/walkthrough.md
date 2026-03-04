# Sprint 18 Walkthrough (2026-03-04)

## 当前进度
1. 语义层已具备最小 name resolution 与函数体作用域入口。
2. 本轮新增 builtin 调用参数个数校验，解析后可直接阻断明显非法调用。
3. 本轮新增函数体 return 约束（禁止 void-return、要求可达 value-return）。
4. 自举回归已覆盖 builtin arity 与 function-return 正/负路径。

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

## 验证方式
1. 运行 `./scripts/run_checks.sh`。
2. 运行 `./scripts/run_lency_checks.sh`。

## 未尽事宜
1. 类型一致性校验仍未落地（int/bool/string/float）。
