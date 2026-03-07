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
- [x] 基础类型一致性校验（最小 `int/bool/string/float`，覆盖赋值/一元/二元/逻辑）
- [x] 用户函数最小 arity 校验（含“先调用后声明”）
- [x] 用户函数类型签名校验（参数类型 + 返回类型）

### 待完成
- [x] 自定义类型签名接入（`T_IDENTIFIER` 类型名的参数/返回类型解析与校验）

---

## Sprint 17: 自举 - Parser (收尾中)

### 目标
实现递归下降解析器，将 Token 流转换为 AST，并维持可测试的模块化结构。

### 收尾项
- [ ] AST 定义补全（Type representation 等未落地部分）
- [ ] 声明解析扩展（`func/struct/impl` 最小骨架）
- [x] Parser 错误恢复同步点（当前仍偏 fail-fast）

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

*更新时间: 2026-03-07*

### 今日增量（2026-03-05）
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
13. resolver 新增最小类型一致性检查：字面量与局部变量类型跟踪（`int/bool/string/float`），并在赋值/算术/比较/逻辑/一元运算做一致性约束。
14. `test_cases` + `test_entry` 新增 Step 16 类型一致性回归（正例 + 赋值/算术/逻辑负例）。
15. 为兼容现有自举 runtime pointer-as-value 链路，`arg_at/int_to_string/float_to_string/bool_to_string` 在 resolver builtin 返回类型中先按 `unknown` 处理，避免误伤现有运行回归。
16. parser 新增最小函数声明骨架（`int/string/bool/void/float name(...) { ... }`），参数类型写入 AST（`param_kinds`）。
17. resolver 新增用户函数 arity 预扫描与调用校验，覆盖“先调用后声明”路径。
18. `test_cases` + `test_entry` 新增 Step 17 用户函数 arity 正/负例回归，并通过 `run_lency_checks.sh` 全链路验证。
19. token/lexer 新增 `float` 关键字支持（`T_FLOAT`），函数签名语法补齐 `float`。
20. AST 函数节点新增 `param_kinds`，parser 函数声明保留参数类型信息。
21. resolver 新增用户函数类型签名预扫描与调用参数类型校验，并新增 return 返回类型校验。
22. `test_cases` + `test_entry` 新增 Step 18（用户函数签名正/负例），全链路通过。
23. resolver 模块拆分为 `resolver.lcy + resolver/core.lcy + resolver/expr.lcy`，满足单文件行数约束。
24. parser 函数声明签名接入 `T_IDENTIFIER`（自定义类型名），并增加安全判定避免误判普通表达式语句。
25. AST 函数声明节点新增 `return_type_name/param_type_names`，保留签名中的类型名信息供 resolver 校验。
26. resolver 新增签名类型校验：`T_IDENTIFIER` 类型名需已声明（未声明报 `unknown type in signature`）。
27. `test_cases` + `test_entry` 新增 Step 19（自定义类型签名正/负例），`check-lency` 全链路通过。
28. parser 新增最小错误恢复同步点（declaration 级 recover），不再首错即终止；新增回归用例验证“前一条语句错误后仍可继续解析后续语句”。

### 今日增量（2026-03-06）
1. AST 新增最小 `TypeRef` 结构表示，为后续类型节点落地预留稳定入口。
2. parser 声明层新增 `struct` 最小骨架解析：支持 `struct Name { ... }` 入 AST（当前成员体先跳过解析）。
3. AST 新增 `STMT_STRUCT` 与 `make_stmt_struct`，printer 新增 `(struct Name)` 文本输出。
4. resolver 新增类型声明预扫描 `preload_user_type_declarations`：在函数签名校验前先注册 `struct` 类型名。
5. resolver 语句分派新增 `STMT_STRUCT` 分支（声明节点不报 unsupported）。
6. `test_cases` + `test_entry` 新增 Step 20（type declaration skeleton），覆盖 parse AST 形态与“struct + 自定义类型签名”正例。
7. `cargo run -p xtask -- check-lency` 全链路通过。
8. parser 声明层新增 `impl` 最小骨架解析：支持 `impl Type { ... }` 入 AST（当前成员体先跳过解析）。
9. AST 新增 `STMT_IMPL` 与 `make_stmt_impl`，printer 新增 `(impl Type)` 文本输出。
10. resolver 语句分派新增 `STMT_IMPL` 分支（声明节点不报 unsupported）。
11. `test_cases` + `test_entry` 新增 Step 21（impl declaration skeleton），覆盖 parse AST 形态与“struct + impl + 自定义类型签名”正例。
12. 再次执行 `cargo run -p xtask -- check-lency`，全链路通过。
13. `impl` 声明解析升级为“成员函数骨架”模式：`impl Type { int f(...) { ... } }` 可解析为 `impl` 节点下的函数声明列表。
14. `AST printer` 的 `impl` 输出改为包含成员列表：`(impl Type [...])`。
15. `test_cases` + `test_entry` 新增 Step 22，固定校验 `impl` 成员函数骨架 AST 形态（包含 `(func value/0 ...)`）。
16. 再次执行 `cargo run -p xtask -- check-lency`，全链路通过。

### 今日增量（2026-03-07）
1. resolver `STMT_IMPL` 接入最小语义约束：`impl` 目标类型必须可解析，否则报 `unknown type in impl target`。
2. resolver `STMT_IMPL` 接入同一 `impl` 内方法名去重校验：重复方法报 `duplicate method in impl`。
3. `impl` 成员函数体复用既有函数语义路径（参数/返回签名与 return 约束）。
4. `test_cases` + `test_entry` 新增 Step 23（impl 语义正/负例）：目标类型不存在、方法重名负例。
5. Windows CI 修复：`scripts/win/setup-dev.ps1` 支持 `-SearchRoots` 根目录直接命中 LLVM 前缀，并放宽 LLVM 目录名匹配（兼容 `clang+llvm-*`）。
6. Windows CI 排障增强：`.github/workflows/tests.yml` 新增 `Debug LLVM archive layout` 步骤，输出解压目录结构与 `llvm-config*.exe` 候选路径。
