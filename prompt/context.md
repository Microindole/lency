# Lency 项目核心上下文 (Agent Root)

> [!IMPORTANT]
> **外科手术式技能加载协议 (Surgical Skill Loading Protocol)**：
> 为保持注意力高度集中并节省 Token，你必须依任务类型**仅加载**对应的子技能文件：
> - **架构决策/规范评审**：仅阅读 `prompt/skills/architect/SKILL.md`
> - **流水线管理/状态同步**：仅阅读 `prompt/skills/management/SKILL.md`
> - **编译器开发/故障排查**：首先阅读 `prompt/skills/compiler/SKILL.md` 的路由图，然后**仅加载**相关的 Crate 指南（如 `crates/sema.md`）。
> - **生态/工具链/LSP**：仅阅读 `prompt/skills/tooling/SKILL.md`
> - **文档风格**：避免使用 emoji，用纯文本标记（如 [DONE]、[WIP]、[TODO]）替代。

## 项目概述
**Lency** 是一门静态类型、编译型语言，基于 LLVM 实现。设计哲学：「简洁、规范、清晰」，无“黑魔法”。

## 核心设计准则 (见 Architect Skill)
- **空安全**：默认非空，`T?` 表示可空。
- **无分号**：使用 `{}` 结构，行尾无 `;`。
- **显式优先**：禁止隐式类型转换和复杂的推理。

crates/          # 编译器内核 (Rust)
prompt/          # Agent 职能中心
  context.md     # 入口：项目地图与注意力协议
  sprint/        # 战术层：当前状态与计划 (status.md)
  skills/        # 工具层：模块化 SOP (Surgical Skills)
assets/          # 战略层：语言规范与设计哲学 (规范、蓝图)
lib/             # Lency 标准库 (.lcy)
tests/           # 集成测试集
scripts/         # 自动化检查与开发工具
editors/         # IDE 插件与工具链
```

1. **启动**：优先阅读 `prompt/context.md` 获取地图。
2. **同步**：阅读 `prompt/sprint/status.md` 确定当前战术目标。
3. **执行**：按需加载 `prompt/skills/` 下的子技能。
4. **验证**：运行 `./scripts/run_checks.sh` 与 `./scripts/run_lency_checks.sh`。
5. **交付**：必须更新 `prompt/sprint/status.md` 及 `prompt/context.md` (如有架构/状态变更)。
6. **存档**：将最新的 task/implementation_plan/walkthrough 同步到 `prompt/artifacts/`，确保跨会话可追溯。

## 当前编译器状态 (完成度: ~65%)
- [DONE] 基础语法、泛型、Null安全、Enum、Vec、HashMap、Iterator
- [DONE] 统一诊断系统 (lency_diagnostics)
- [DONE] Result/Option 方法全量支持 (is_ok, unwrap, unwrap_or, expect等)
- [DONE] panic 机制强化（支持动态消息、文件行号追踪）
- [DONE] String 格式化 -- `format(string, Vec<string>)` 内置函数
- [DONE] 标准库清理 -- core 瘦身、去重、Iterator 统一、string→str 重命名
- [DONE] 标准库增强 -- 24 个新函数 (str/io/collections/math/char)
- [DONE] 修复 CI 定时任务失败 (升级 bytes 依赖解决 cargo audit 漏洞)
- [DONE] CLI 产物路径优化 -- `lencyc compile/build` 支持 `--out-dir`，可将编译结果集中到指定目录
- [WIP] 自举 Lexer & Parser 重新开始
  - 已完成 Token、Lexer 和 Parser 基础骨架，并集成到了 Github CI (`tests.yml` 中的 `self-hosted-tests`)
  - Parser 已支持表达式优先级链（assignment/logical/comparison/arithmetic/unary/primary）与 `var/if/while/block/return` 语句
  - 2026-03-03 增量：`primary` 已补齐 `false` 字面量解析，并在 `lencyc/driver/test_entry.lcy` 加入布尔逻辑用例
  - 2026-03-03 工具链修正：`run_checks.sh` / `run_lency_checks.sh` 改为无参数固定职责；前者仅 Rust 侧检查，后者仅 Lency 侧检查；作用域规则固定为 `lib/` 同时属于 rust/lency，`tests/example/` 归 lency，`tests/integration/` 归 rust
  - 2026-03-03 CI 策略修正：Github Actions 先做变更路径判定，再按 rust/lency 作用域触发对应 job；未命中改动的侧不运行，减少无效 CI 消耗
  - 已提供 AST 可观测性：`expr_to_string` / `stmt_to_string` 已在 `lencyc/driver/test_entry.lcy` 接入
  - 2026-03-02 基线验证：`./scripts/run_checks.sh` 与 `./scripts/run_lency_checks.sh` 均通过
  - **关键规则**：必须“一步步一点点的新增”关键字和语法特性，每新增一个特性**必须**立即运行 `./scripts/run_lency_checks.sh` 进行验证，防止旧版本 Rust Lency 编译器的隐藏 Bug 导致 LLVM 报错。
---
详细设计参考: [design_spec.md](../assets/design_spec.md), [Lency.txt](../assets/Lency.txt)
