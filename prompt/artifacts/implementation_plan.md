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

## 下一步 (按优先级)
1. 类型一致性最小闭环
   - 先覆盖赋值与二元表达式上的最小类型冲突检查（int/bool/string/float）。
2. 调用语义扩展
   - 非 builtin 函数签名来源与调用校验（依赖函数声明语义接入）。

## 质量门禁
每次改动结束必须执行：
1. `./scripts/run_checks.sh`
2. `./scripts/run_lency_checks.sh`

## 当前技术债
1. 语义诊断仍是 `print` 文本，未统一 Reporter。
2. 非 builtin 函数目前缺少签名源，调用校验仅覆盖 builtin。
