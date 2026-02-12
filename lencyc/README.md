# Lency Self-Hosted Compiler

此目录 (`lencyc/`) 将存放自举编译器 (Lency 编写的 Lency 编译器) 的源代码。这是与 Rust 实现 (`crates/`) 并列的顶级目录。

## 目录结构规划 (Sprint 16+)

*   `token.lcy`: Token 定义
*   `lexer.lcy`: 词法分析器实现
*   `ast.lcy`: 抽象语法树定义
*   `parser.lcy`: 语法分析器实现
*   `sema/`: 语义分析模块
*   `codegen/`: 代码生成模块
*   `driver.lcy`: 编译器驱动入口

当前状态：**Sprint 16 准备中**
