# Lency Project Context for AI Agents

> [!TIP]
> **Agent 任务导航指南**：为节省上下文长度，请根据您的任务类型优先阅读指定区域：
> - **编译器核心开发**：关注 `crates/` (Lexer, Parser, Codegen, Runtime)。
> - **语言标准库/运行时**：关注 `lib/` (.lcy) 和 `crates/lency_runtime/` (FFI)。
> - **编辑器/IDE 支持**：仅关注 `editors/` (VS Code 扩展等)。
> - **宏观设计与文档**：参考 `assets/` (Spec) 和 `docs/`。

## 项目概述
**Lency** 是一门静态类型、编译型语言，编译到 LLVM IR。设计哲学：「简洁、规范、清晰」，无黑魔法。

## 核心规则
- **无分号**：行尾不写 `;`，必须用 `{}`
- **非空安全**：默认非空，`T?` 表示可空
- **泛型单态化**：无运行时泛型

## 目录结构
```
crates/          # Rust 编译器组件
  lency_syntax/  # 词法/语法分析
  lency_sema/    # 语义分析
  lency_codegen/ # LLVM 代码生成
  lency_runtime/ # C FFI 运行时
editors/         # 编辑器支持 (VS Code 扩展等)
lib/             # Lency 标准库
tests/           # .lcy 集成测试
```

## 关键命令
```bash
./scripts/run_checks.sh   # 全量检查（必须通过）
cargo build --release     # 构建编译器
```

## 当前状态
- ✅ 基础语法、泛型、Null安全、Enum、Vec、HashMap、Iterator、String intrinsics
- ✅ VS Code 语法高亮支持 (.lcy)
- ⚠️ Result 方法（需实现）、panic 机制（未实现）

---
详细设计参考: [design_spec.md](./design_spec.md), [Lency.txt](./Lency.txt)
