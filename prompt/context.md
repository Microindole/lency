# Lency 项目核心上下文 (Agent Root)

> [!IMPORTANT]
> **外科手术式技能加载协议 (Surgical Skill Loading Protocol)**：
> 为保持注意力高度集中并节省 Token，你必须依任务类型**仅加载**对应的子技能文件：
> - **架构决策/规范评审**：仅阅读 `prompt/skills/architect/SKILL.md`
> - **流水线管理/状态同步**：仅阅读 `prompt/skills/management/SKILL.md`
> - **编译器开发/故障排查**：首先阅读 `prompt/skills/compiler/SKILL.md` 的路由图，然后**仅加载**相关的 Crate 指南（如 `crates/sema.md`）。
> - **生态/工具链/LSP**：仅阅读 `prompt/skills/tooling/SKILL.md`

## 项目概述
**Lency** 是一门静态类型、编译型语言，基于 LLVM 实现。设计哲学：「简洁、规范、清晰」，无“黑魔法”。

## 核心设计准则 (见 Architect Skill)
- **空安全**：默认非空，`T?` 表示可空。
- **无分号**：使用 `{}` 结构，行尾无 `;`。
- **显式优先**：禁止隐式类型转换和复杂的推理。

crates/          # 编译器内核 (Rust)
prompt/          # Agent 职能中心
  context.md     # 🚀 入口：项目地图与注意力协议
  sprint/        # 🏃 战术层：当前状态与计划 (status.md)
  skills/        # 🛠️ 工具层：模块化 SOP (Surgical Skills)
assets/          # 🏛️ 战略层：语言规范与设计哲学 (规范、蓝图)
lib/             # Lency 标准库 (.lcy)
tests/           # 集成测试集
scripts/         # 自动化检查与开发工具
editors/         # IDE 插件与工具链
```

1. **启动**：优先阅读 `prompt/context.md` 获取地图。
2. **同步**：阅读 `prompt/sprint/status.md` 确定当前战术目标。
3. **执行**：按需加载 `prompt/skills/` 下的子技能。
4. **验证**：运行 `./scripts/run_checks.sh --fast`。
5. **交付**：及时更新 `prompt/sprint/status.md`。

## 当前编译器状态
- ✅ 基础语法、泛型、Null安全、Enum、Vec、HashMap、Iterator
- ✅ 统一诊断系统 (lency_diagnostics)
- ✅ Result/Option 方法全量支持 (is_ok, unwrap, unwrap_or, expect等)
- ⚠️ panic 机制（待强化，目前支持基础 exit(1)）

---
详细设计参考: [design_spec.md](../assets/design_spec.md), [Lency.txt](../assets/Lency.txt)
