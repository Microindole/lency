# 贡献与维护规则

## 改动原则

- 当前主线是 Lency 自举编译器；Rust 母体以稳定为先。
- 不以“把 Rust 功能全部搬进 selfhost”为目标，只实现编译器自身和下一阶段验证所需的最小闭包。
- 改动应局部化，不借任务之名重排无关代码。
- 没有明确失败路径时，不添加预防性兜底或静默回退。
- 不使用新的 `TYPE_UNKNOWN` 路径掩盖类型错误。
- 注释说明意图、约束和非直观边界，不复述代码。
- 同类分派持续增长时按领域拆分，避免单文件堆积 `if-else`。

## 测试归属

| 改动 | 优先测试位置 |
|---|---|
| Rust 母体语法、语义、后端 | `tests/integration/` 与对应 crate 单元测试 |
| selfhost parser/sema | `tests/example/selfhost/driver/steps/` |
| selfhost LIR | `tests/example/lir/` |
| selfhost 生成程序行为 | `tests/example/runtime/` |
| import/module | `tests/example/modules/` |

任何可观察行为变化、缺陷修复或重构都必须有对应回归。测试应尽量小，并能直接说明失效边界。

## 验证命令

Lency 主线改动：

```powershell
cargo run -p xtask -- check-lency
```

涉及 Rust 母体：

```powershell
cargo run -p xtask -- check-rust
```

涉及自举收敛：

```powershell
cargo run -p xtask -- bootstrap-check
```

`cargo run -p xtask -- auto-check` 适合日常按改动范围派发，但提交前应根据实际影响显式选择上述门禁。

## 文档规则

- 稳定的语言原则只写入 `docs/design.md`。
- 当前实现状态只写入 `docs/development/status.md`。
- 工具命令只写入 `docs/tools/scripts.md`。
- 已完成的逐提交流水账由 Git 历史承担，不再维护 walkthrough 或 Sprint 完成项长清单。
- 文档中的“支持”必须注明是 Rust 母体、selfhost 前端，还是 selfhost 端到端支持。

### 源码目录 README

- `crates/` 与 `lencyc/` 中，每个具有独立职责、入口或设计约束的逻辑模块至少维护一份就近的 `README.md`。
- README 说明模块职责、主要入口、文件和子目录分工、上下游数据流、关键边界与检查方式。
- 只为控制文件体积而拆出的目录不重复建 README，由最近的父模块文档统一说明。
- README 是源码地图，不复制 `docs/` 中的语言规范，也不记录逐提交完成日志。
- 新增、移动或删除模块和关键入口时，必须同步更新最近的 README。
