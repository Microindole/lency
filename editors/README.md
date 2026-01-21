# Lency 编辑器支持 (Professional Support)

本目录包含了 Lency 语言在现代编辑器（特别是 VS Code / Cursor / Antigravity）中的核心开发支持。

## 🔹 核心特性

- **高级语义感知**: 
    - **跳转到定义 (F12 / Ctrl+Click)**: 快速跳转至函数、结构体、特征的定义处。
    - **重命名 (F2)**: 一键安全重命名变量、函数和类型，自动同步更新文件内引用。
    - **智能补全 (Completions)**: 提供带有 Markdown 文档说明的关键字和内置函数建议。
- **智能辅助**:
    - **精准签名助手**: 输入函数参数时动态高亮当前参数原型。
    - **悬停预览**: 鼠标悬停显示内置函数文档。
    - **代码高亮**: 基于 TextMate 语法的深度词法染色。
- **工程化工具**:
    - **自动格式化 (Alt+Shift+F)**: 基于缩进规则的一键美化。
    - **编译器联动**: 直接捕捉 `.lcy` 编译错误并在“问题”面板展示。
    - **构建任务**: 支持 `Ctrl+Shift+B` 快速编译当前文件。

---

## 🛠️ 快速开始

### 1. 开发与调试 (推荐)
如果您正在开发 Lency 语言，推荐使用内置的开发模式脚本：
```bash
./scripts/dev_ide.sh
```
该脚本会自动执行：
1. 编译最新的 TypeScript 扩展源码。
2. 以扩展开发模式启动主编辑器，并自动加载本地插件。

### 2. 手动安装/构建
如果您想手动管理插件，请进入 `vscode` 目录：
```bash
cd editors/vscode
npm install       # 安装依赖
npm run build     # 编译产生 dist/extension.js
```
编译完成后，将该文件夹路径添加到 VS Code 的扩展开发路径中，或建立软链接到扩展目录。

---

## 🏗️ 项目架构

- `vscode/src/`: TypeScript 源码目录。
- `vscode/dist/`: 编译后的产物（运行必备）。
- `vscode/syntaxes/`: TextMate 语法定义网格。
- `vscode/snippets/`: 代码片段预设。
- `FUTURE_PLAN.md`: 涵盖 LSP 与语义分析演进的长远技术路线图。

---

## ✅ 自动化检查 (CI)
本项目已集成编辑器插件的质量检查。您可以运行：
```bash
./scripts/run_checks.sh
```
这会触发 `scripts/check_editor.sh` 对插件进行类型检查和构建验证。

---

*Lency Professional Support - 致力于提供最纯净、高效的语言开发体验。*
