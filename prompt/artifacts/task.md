# Sprint 16 Tasks: Bootstrap - Lexer

- [ ] **Token 定义**
    - [ ] `TokenType` enum
    - [ ] `Token` struct
    - [ ] `to_string` 方法

- [ ] **String Helper** (如果标准库缺少)
    - [ ] `is_digit(char)`
    - [ ] `is_alpha(char)`
    - [ ] `is_alphanumeric(char)`

- [ ] **Lexer 结构**
    - [ ] `struct Lexer`
    - [ ] `advance()`, `peek()`
    - [ ] `skip_whitespace()`

- [ ] **Scanner 逻辑**
    - [ ] 单字符符号
    - [ ] 运算符 (`==`, `!=` 等)
    - [ ] 字符串 (`"`)
    - [ ] 数字 (int/float)
    - [ ] 标识符 & 关键字

- [ ] **验证**
    - [ ] 编写 `tests/integration/bootstrap/lexer_test.lcy`
    - [ ] 运行 `./scripts/run_checks.sh`
