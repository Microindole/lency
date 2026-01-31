---
name: management
description: 项目节奏控制与质量门禁。
---

# 项目管理 (Management) - 协议规范

## 1. 任务更新协议 (Protocol)
- **开启任务**: 核对 `tests/integration/` 下的相关 `.lcy`。
- **同步状态**: 
    - 统计 `@expect-error:TODO` (待实现) 与 `FIXME` (Bug)。
    - 量化“自举准备度”(当前目标 63%)。
    - 更新 [status.md](./resources/status.md)。

## 2. 质量门禁 (Quality Gates)
> [!IMPORTANT]
> 交付前必须通过：
> 1. `./scripts/run_checks.sh --fast` (格式/Clippy/单元测试)。
> 2. `scripts/run_lcy_tests.sh` (集成测试零回归)。

## 3. 测试标记
- **✅ Pass**: 结果符合预期。
- **📋 TODO**: 语义已定，后端未连通。
- **🐛 FIXME**: 已知编译器 Bug，最高优先级。

## 4. 会话移交协议 (Handover)
每次对话结束前，必须在 `walkthrough.md` 包含：
- **当前进度**: 刚完成的 P0 任务。
- **未尽事宜**: 下一个 Agent 接手时应第一步执行的操作。
- **阻止项**: 目前阻塞进度的具体 Crate 或 Bug。

## 5. 审计清单 (Audit)
- [ ] `prompt/context.md` 是否反映了最新的 `crates/` 结构？
- [ ] 所有新增加的 `.lcy` 测试是否已在 `run_lcy_tests.sh` 中注册或扫描？
- [ ] `FIXME` 数量是否通过代码提交有所减少？

## 6. 自我演进协议 (Self-Evolution)
作为 Agent，你拥有**持续优化此技能系统**的义务：
- **坑位捕获**: 发现新的技术陷阱或 Bug 根源时，立即更新对应 Crate 的 `Pitfalls`。
- **模式提取**: 成功执行复杂任务后，将其抽象为可复用的 `Recipe` (菜谱)。
- **规范修正**: 若发现现有规范与最新自举进度冲突，报请用户批准后更新。

---
[Sprint 计划](./resources/plan_15.md) | [进度表](./resources/status.md)
