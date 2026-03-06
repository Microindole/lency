# Lency 编辑器支持

本目录提供 Lency 在 VS Code 侧的基础开发支持。当前定位是“可用、可回归、边界清晰”，不是完整语义 IDE。

## 当前可用能力

- 语法高亮：TextMate 规则（`syntaxes/lency.tmLanguage.json`）。
- 代码片段：基础模板（`snippets/lency.code-snippets`）。
- 本地语义辅助（单文件）：
  - 文档符号（大纲）
  - 悬停内建文档
  - 标识符高亮
  - 基础定义跳转
  - 基础重命名
  - 签名提示（内建函数）
  - 基础格式化（基于花括号缩进）
- LSP 启动降级：
  - 检测到 `lency_ls` 时，自动启动语言服务器。
  - 未检测到时，自动启用本地降级能力并提示。
- 本地诊断降级：LSP 不可用时，启用括号匹配诊断。

## 明确边界

- `FIXME`: 现有格式化器未处理字符串/注释中的花括号，复杂行可能误缩进。
- `TODO`: 定义跳转、重命名目前是单文件模型，尚未跨文件符号索引。
- `TODO`: 诊断目前只有本地括号匹配；类型/语义诊断应由 LSP 提供。

## 开发与调试

```bash
npm --prefix editors/vscode run dev:ide
```

该脚本用于本地扩展开发调试。

## 手动构建

```bash
cd editors/vscode
npm install
npm run build
```

## 如何开启

1. 启动扩展开发宿主：
```bash
npm --prefix editors/vscode run dev:ide
```
2. 在新开的 VS Code/Cursor 窗口打开任意 `.lcy` 文件，状态栏看到语言为 `Lency` 即已启用扩展。
3. 看左下角状态栏模式：
   - `Lency: LSP`：语言服务已连接（推荐）。
   - `Lency: Fallback`：仅本地降级能力（你现在说的“只有基本彩色”通常就在这个模式）。
4. 要启用 LSP（非 fallback），确保 `lency_ls` 可执行文件可被找到：
```bash
cargo build -p lency_ls
```
5. 如需手工指定路径，在编辑器设置中配置：
```json
"lency.serverPath": "${workspaceFolder}/target/debug/lency_ls"
```
Windows 示例：
```json
"lency.serverPath": "D:\\works\\lency\\target\\debug\\lency_ls.exe"
```
6. 现在支持热更新：修改 `lency.serverPath` 后会自动重连并切换模式，不再要求手动 Reload Window。

## 故障排查

1. 只有语法高亮，没有跳转/重命名  
看左下角是否为 `Lency: Fallback`。若是，先执行：
```bash
cargo build -p lency_ls
```
并设置 `lency.serverPath`。

2. `lency.serverPath` 已设置但仍是 `Fallback`  
确认路径存在且可执行：
```bash
ls -l target/debug/lency_ls
```
PowerShell:
```powershell
Get-Item .\target\debug\lency_ls.exe
```

3. 修改 `lency.serverPath` 后模式没切换  
查看 `Output -> Lency Language Server`，确认重连日志；若无日志，检查扩展是否在运行（Running Extensions）。

## 目录结构

- `vscode/src/`: 扩展源码。
  - `core/`: 语言元数据、LSP 启动、fallback 诊断等基础设施模块。
  - `providers/`: definition/completion/rename 等能力模块。
  - `extension.ts`: 仅负责装配与生命周期。
- `vscode/dist/`: 编译产物。
- `vscode/syntaxes/`: 语法高亮定义。
- `vscode/snippets/`: 代码片段。
- `docs/FUTURE_PLAN.md`: 编辑器能力演进路线。

## 检查

```bash
npm --prefix editors/vscode run check:all
```

Editors 任务只运行这一个 Node 入口，不和自举/Rust 主流程混跑。
