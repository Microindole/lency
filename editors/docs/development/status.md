# 编辑器实现状态

更新：2026-09-21

## 当前结论

- VS Code 扩展已具备语法高亮、代码片段、基础格式化和本地 fallback provider。
- 扩展会优先启动 `lency_ls`；找不到服务端时切换到 fallback，并在状态栏显示当前模式。
- `lency.serverPath` 支持 `${workspaceFolder}`，配置变化后会自动重连。
- 当前开发优先级低于 Lency 自举主线；只修复真实的编辑器回归，不扩张独立语义子系统。

## 已具备的能力

- TextMate 语法高亮和 Lency 文件图标。
- 常用语法的 snippet。
- 文档符号、内建函数悬停、标识符高亮和签名提示。
- fallback 定义跳转和重命名，可扫描工作区中的 `.lcy` 文件。
- 忽略字符串和行注释中花括号的基础格式化。
- fallback 括号、未闭合字符串和全角标点诊断。
- VS Code 扩展打包与 `evX.Y.Z` 发布流程。

## 已知边界

- fallback 的跨文件定义和重命名是文本启发式，不具备作用域和类型级精度。
- `TODO`：类型、引用查找、完整语义诊断和增量分析继续收敛到 `crates/lency_ls`。
- 编辑器不判定 selfhost 能力；自举实现状态以 [docs/development/status.md](../../../docs/development/status.md) 为准。

## 验收入口

```powershell
npm --prefix editors/vscode run check:all
```

该入口覆盖 TypeScript 构建、provider 回归、开发宿主启动检查和 Editors 代码质量检查。
