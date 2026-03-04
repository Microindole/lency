# Sprint 状态总结

## Sprint 17: 自举 - Parser (进行中)

**工作记录**: [task](../artifacts/task.md) | [implementation_plan](../artifacts/implementation_plan.md) | [walkthrough](../artifacts/walkthrough.md) | [详细计划](plan_17.md)

### 目标
实现一个递归下降解析器 (Recursive Descent Parser)，将 Token 流转换为 AST。

### 待完成
- [ ] AST 定义 (Enum/Struct) - `lencyc/syntax/ast.lcy`（已覆盖 Expr/Stmt 基础节点，含 `return`）
- [x] Parser 基础架构 - `lencyc/syntax/parser.lcy`
- [ ] Expression Parsing (优先级, Pratt/Recursive)（已支持 assignment/logical/comparison/arithmetic/unary/primary，含 `true/false/string-literal/int-float-literal` 字面量）
- [ ] Statement/Declaration Parsing（已支持 var/if/while/for/block/return/return-void/break/continue/expr）
- [x] AST Printer (Debug验证)
- [x] Parser 模块化拆分（D&D）：`lencyc/syntax/parser/{expr,stmt,decl}.lcy`
- [x] AST 模块化拆分：`lencyc/syntax/ast/{expr,stmt,printer}.lcy`

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

### 优先级 1: Sprint 17 -- Parser Implementation

### 优先级 2: Sprint 18 -- Semantic Analysis (Name Resolution)

---

## 统计
| 指标 | 值 |
|------|-----|
| 测试通过 | 69 (.lcy) + Rust unit tests |
| 自举组件 | Lexer (Done), Parser (WIP) |
| 自举准备度 | ~98% |

*更新时间: 2026-03-04*

### 今日增量（2026-03-04）
1. 自举 Lexer 新增字符串字面量扫描：`"` 开始与结束，产出 `T_STRING_LITERAL`。
2. 自举 Parser `primary` 新增字符串字面量分支，AST 走 `EXPR_LITERAL` 统一路径。
3. 自举回归新增字符串正例：`var msg = "hello"` 与 `print("done")` 的 AST 断言。
4. 自举回归新增字符串负例：未闭合字符串字面量应被 parser 拒绝。
5. 自举 Lexer `number()` 新增浮点扫描：支持 `digits '.' digits`（仍归类 `T_NUMBER`）。
6. 自举回归新增浮点正例：`3.14`、`0.5`；新增浮点负例：`12.`（缺少小数部分）应被 parser 拒绝。

### 今日增量（2026-03-03）
1. 自举 Parser 新增 `break` 语句：
   - 词法：`break` 关键字映射到 `T_BREAK`
   - AST：新增 `STMT_BREAK` 与构造函数
   - 语法：新增 `break_statement`，并限制只能出现在 `while` 块内
   - 调试打印：新增 `(break)` 输出
2. `prompt/context.md` 已重构为“目录地图+职责说明”，移除逐条流水账。
3. 自举 Parser 新增 `continue` 语句，并补充 `break/continue` 循环外非法位置负例测试。
4. 自举 Parser 新增 C 风格 `for` 语句解析（先反糖为 `while` 路径）。
5. 修复 `for` 反糖语义：`continue` 现在会先执行 `increment`，并避免污染嵌套循环的 `continue`。
6. 自举测试扩展：补齐 `for` 非法语法负例（缺 `var` / `;` / `{`）与嵌套循环重写安全性断言。
7. 新增最小 `name resolution` 骨架（`lencyc/sema/{symbol,scope,resolver}.lcy`），并在 `test_entry` 中加入 smoke test。
8. 为 resolver 新增负例断言：未定义变量、未定义赋值、同作用域重复定义。
9. `for` 初始化扩展：从仅 `var` 扩展为支持表达式初始化（`for i = ...;`）。
10. resolver 测试扩展：新增“块退出后不可见”负例与“作用域遮蔽合法”正例。
11. parser 表达式链扩展：新增 `call/member` 解析与 AST 节点（`EXPR_CALL` / `EXPR_GET`）。
12. parser 架构收敛：抽出 `lower_for_statement`，集中管理 for 反糖规则。
13. resolver 扩展：新增函数体级入口 `resolve_function_body(params, body)`，并补函数样式作用域测试。
14. 自举回归集结构化：新增 `lencyc/driver/test_cases.lcy` 管理测试源码，`test_entry` 仅负责编排与断言。
