# Lency Project Context for AI Agents

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
- ⚠️ Result 方法（需实现）、panic 机制（未实现）

---
详细设计参考: [design_spec.md](./design_spec.md), [Lency.txt](./Lency.txt)
