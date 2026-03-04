# Sprint 17 Tasks: Bootstrap - Parser

- [ ] **AST 定义** (`lencyc/syntax/ast.lcy`)
    - [x] `Expr` 基础节点 (Binary, Unary, Literal, Variable, Assign, Logical)
    - [x] `Stmt` 基础节点 (If, While, VarDecl, Block, Return, Expr)
    - [ ] `Type` representation

- [x] **Parser 基础** (`lencyc/syntax/parser.lcy`)
    - [x] `struct Parser`
    - [x] `match`, `consume`, `check` helper methods
    - [ ] Error synchronization logic

- [ ] **Expression Parsing**
    - [x] 递归优先级链 (`assignment -> or -> and -> equality -> comparison -> term -> factor -> unary -> primary`)
    - [x] Leaf nodes (number, bool, identifier, grouping)
    - [x] Infix/Prefix operators
    - [x] 字符串字面量扩展（`T_STRING_LITERAL`，含正/负例回归）
    - [x] 浮点字面量扩展（`digits '.' digits`，沿用 `T_NUMBER`）
    - [ ] 科学计数法字面量扩展（如 `1.2e-3`）

- [ ] **Statement Parsing**
    - [x] `parse_decl` (var)
    - [x] `parse_stmt` (if, while, block, return, expr_stmt)
    - [ ] `func/struct/impl` 声明解析

- [ ] **验证 & 驱动**
    - [x] AST Pretty Printer (`expr_to_string` / `stmt_to_string`)
    - [x] 更新 `lencyc/driver/test_entry.lcy` (覆盖 `return` 解析路径)
    - [x] 运行 `./scripts/run_lency_checks.sh`
    - [x] `lencyc/driver/main.lcy` 串联最小完整自举流程（Read/Lex/Parse/Resolve/Emit）

---

# Completed (Sprint 16)
- [x] Token 定义
- [x] Keywords 映射
- [x] Lexer 核心逻辑
- [x] Driver 验证
