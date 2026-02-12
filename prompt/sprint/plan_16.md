# Sprint 16: 自举 - 词法分析器 (Lexer)

**目标**: 使用 Lency 语言实现一个功能完整的 Lexer，能够解析 Lency 源代码并生成 Token 流。这是自举编译器的第一步。

## 核心任务

### 1. Token 定义
- [ ] 定义 `TokenType` enum (关键字、符号、字面量)
- [ ] 定义 `Token` struct (type, lexeme, line, col)
- [ ] 实现 `Token` 的 `to_string()` 用于调试

### 2. Lexer 基础架构
- [ ] 定义 `Lexer` struct
    - source code (string)
    - position (int)
    - current_char (char/string)
- [ ] 实现基本方法：
    - `new(source)`
    - `advance()`
    - `peek()`
    - `is_at_end()`

### 3. Scanner 实现
- [ ] 处理单字符 Token (`+`, `-`, `*`, `/`, `(`, `)`, `{`, `}`)
- [ ] 处理双字符 Token (`==`, `!=`, `<=`, `>=`)
- [ ] 处理空白字符和注释 (`// ...`)
- [ ] 处理字符串字面量 (`"..."`)
- [ ] 处理数字字面量 (int/float)
- [ ] 处理标识符和关键字

### 4. 驱动程序
- [ ] 编写 `main.lcy` 读取字符串并输出 Token 列表
- [ ] 增加集成测试验证 Lexer 正确性

## 依赖检查
- `string` 操作: `char_at`, `len`, `cmp` (Sprint 15 已完成)
- `Vec` 操作: `push`, `get` (Sprint 14 已完成)
- `Enum` 和 `Match`: (Sprint 13/15 已完成，含 String Match)
- `Result`/`Option`: (Sprint 15 已完成)
- `Char Helpers`: `is_digit`, `is_alpha` (Sprint 15 已完成)

## 风险
- 性能问题（大量字符串拷贝），需要优化 `substr` 使用
- Generic HashMap 暂不可用（需使用替代方案）

## 预计产出
- `lencyc/lexer.lcy`
- `tests/integration/bootstrap/test_lexer.lcy`
