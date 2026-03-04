# Sprint 18 Tasks: Bootstrap - Semantic Analysis

- [x] **Name Resolution 最小闭环** (`lencyc/sema/resolver.lcy`)
  - [x] 变量定义/引用校验（undefined / duplicate / out-of-scope）
  - [x] 作用域遮蔽正例覆盖
  - [x] 函数体局部作用域入口（`resolve_function_body`）

- [x] **Builtin 调用约束** (`lencyc/sema/resolver.lcy`)
  - [x] 预载 builtin 名称
  - [x] 预载 builtin arity
  - [x] 在 `EXPR_CALL` 执行参数个数校验

- [ ] **语义下一步**
  - [x] return 语义约束（value-return / void-return）
  - [ ] 最小类型一致性检查（int / bool / string / float）
  - [ ] 非 builtin 函数签名接入与调用校验

- [x] **验证 & 驱动**
  - [x] `lencyc/driver/test_cases.lcy` 新增 builtin arity 与 function-return 约束正/负例
  - [x] `lencyc/driver/test_entry.lcy` 接入 Step 15 与函数体语义约束回归
  - [x] 运行 `./scripts/run_checks.sh`
  - [x] 运行 `./scripts/run_lency_checks.sh`

---

# Parser 收尾并行项 (Sprint 17)
- [ ] `func/struct/impl` 声明解析最小骨架
- [ ] Parser error synchronization
- [ ] AST 类型表示补全（Type representation）
