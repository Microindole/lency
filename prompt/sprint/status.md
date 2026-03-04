# Sprint 状态总结

## Sprint 18: 自举 - Semantic Analysis (进行中)

**工作记录**: [task](../artifacts/task.md) | [implementation_plan](../artifacts/implementation_plan.md) | [walkthrough](../artifacts/walkthrough.md)

### 目标
在保持自举链路可运行的前提下，逐步补齐语义约束（解析后尽早失败，避免错误进入后端）。

### 已完成
- [x] 最小 name resolution：变量定义/引用检查（undefined / duplicate / out-of-scope / shadowing）
- [x] 函数体作用域入口：`resolve_function_body(params, body)`
- [x] 内建符号 prelude 预载（`arg_count/arg_at/...`）
- [x] 内建函数调用参数个数校验（builtin arity）
- [x] 函数体 return 约束（禁止 void-return，要求可达 value-return）

### 待完成
- [ ] 基础类型一致性校验（最小 `int/bool/string/float`）
- [ ] 非 builtin 函数签名来源与调用校验（待函数声明语义接入）

---

## Sprint 17: 自举 - Parser (收尾中)

### 目标
实现递归下降解析器，将 Token 流转换为 AST，并维持可测试的模块化结构。

### 收尾项
- [ ] AST 定义补全（Type representation 等未落地部分）
- [ ] 声明解析扩展（`func/struct/impl` 最小骨架）
- [ ] Parser 错误恢复同步点（当前仍偏 fail-fast）

### 已完成摘要
- [x] Parser/AST 模块化拆分：`lencyc/syntax/{parser,ast}/...`
- [x] 表达式优先级链、`call/member`、`string/char/int/float/scientific` 字面量
- [x] 语句解析：`var/if/while/for/block/return/break/continue/expr`
- [x] `for` 反糖 + `continue` 增量修正

---

## Sprint 16: 自举 - Lexer [DONE]

### 完成内容
1. **Token 定义** [DONE] -- `lencyc/syntax/token.lcy`
2. **Keyword Mapping** [DONE] -- `lencyc/syntax/keywords.lcy`
3. **Lexer 实现** [DONE] -- `lencyc/syntax/lexer.lcy` (完整支持 String/Number/Symbol)
4. **Driver 验证** [DONE] -- `lencyc/driver/main.lcy`

---

## Sprint 15: 自举准备深化 [DONE]

### 完成内容
1. **Iterator trait 实现** [DONE] -- `VecIterator<T>`
2. **`char_to_string` intrinsic** [DONE]
3. **Struct/Enum 返回类型 codegen** [DONE]
4. **`to_upper`/`to_lower`/`reverse`** [DONE]
5. **Result<T,E> 方法** [DONE]
6. **Option<T> 方法** [DONE]
7. **panic 机制强化** [DONE]
8. **String 格式化** [DONE] -- `format(string, Vec<string>)`
9. **String Pattern Matching** [DONE] -- 支持 `match string`
10. **Lency CLI Fix** [DONE] -- `build` 命令链接修复
11. **Standard Library Char** [DONE] -- `lib/std/char.lcy`

---

## 下一步计划

### 优先级 1: Sprint 18 -- Semantic Analysis（类型/调用/返回约束）

### 优先级 2: Sprint 17 -- Parser 收尾（声明解析最小骨架）

---

## 统计
| 指标 | 值 |
|------|-----|
| 测试通过 | 69 (.lcy) + Rust unit tests |
| 自举组件 | Lexer (Done), Parser (Closeout), Sema (WIP) |
| 自举准备度 | ~98% |

*更新时间: 2026-03-04*

### 今日增量（2026-03-04）
1. 自举 Lexer 新增字符串字面量扫描：`"` 开始与结束，产出 `T_STRING_LITERAL`。
2. 自举 Parser `primary` 新增字符串字面量分支，AST 走 `EXPR_LITERAL` 统一路径。
3. 自举回归新增字符串正例：`var msg = "hello"` 与 `print("done")` 的 AST 断言。
4. 自举回归新增字符串负例：未闭合字符串字面量应被 parser 拒绝。
5. 自举 Lexer `number()` 新增浮点扫描：支持 `digits '.' digits`（仍归类 `T_NUMBER`）。
6. 自举回归新增浮点正例：`3.14`、`0.5`；新增浮点负例：`12.`（缺少小数部分）应被 parser 拒绝。
7. 自举主入口 `lencyc/driver/main.lcy` 完成最小流水线串联：读取源码、词法、语法、语义、AST 文本产物输出。
8. 新增 `lencyc/driver/pipeline_sample.lcy` 作为主入口默认样例输入，避免依赖尚未实现的函数声明解析。
9. `scripts/run_lency_checks.sh` 新增对 `lencyc/driver/main.lcy` 的编译、运行与产物校验步骤。
10. 自举最小“完整流程”可执行：`lencyc_main` 运行后可生成 `lencyc_selfhost_ast.txt`。
11. resolver 新增 builtin 参数个数校验：对 `arg_count/arg_at/write_string/...` 固定签名做调用 arity 检查，并补充正/负例回归。
12. resolver 新增函数体 return 约束：value-return 函数禁止 `return` 空值，且要求函数体可达 value-return；`test_entry` 已补齐正/负例回归。
