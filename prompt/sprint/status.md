# Sprint 状态总结

## Sprint 17: 自举 - Parser (进行中)

**工作记录**: [task](../artifacts/task.md) | [implementation_plan](../artifacts/implementation_plan.md) | [walkthrough](../artifacts/walkthrough.md) | [详细计划](plan_17.md)

### 目标
实现一个递归下降解析器 (Recursive Descent Parser)，将 Token 流转换为 AST。

### 待完成
- [ ] AST 定义 (Enum/Struct) - `lencyc/syntax/ast.lcy`（已覆盖 Expr/Stmt 基础节点，含 `return`）
- [x] Parser 基础架构 - `lencyc/syntax/parser.lcy`
- [ ] Expression Parsing (优先级, Pratt/Recursive)（已支持 assignment/logical/comparison/arithmetic/unary/primary，含 `true/false` 字面量）
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

*更新时间: 2026-03-03*

### 今日增量（2026-03-03）
1. 自举 Parser 新增 `break` 语句：
   - 词法：`break` 关键字映射到 `T_BREAK`
   - AST：新增 `STMT_BREAK` 与构造函数
   - 语法：新增 `break_statement`，并限制只能出现在 `while` 块内
   - 调试打印：新增 `(break)` 输出
2. `prompt/context.md` 已重构为“目录地图+职责说明”，移除逐条流水账。
3. 自举 Parser 新增 `continue` 语句，并补充 `break/continue` 循环外非法位置负例测试。
4. 自举 Parser 新增 C 风格 `for` 语句解析（先反糖为 `while` 路径）。
