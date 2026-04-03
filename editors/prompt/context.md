# Editors 子上下文入口

## 0. 最高准则
- 设计哲学仍以 `assets/Lency.txt`、`assets/design_spec.md` 为准。
- 本文件仅记录 `editors/` 相关目标、边界与检查约束。

## 1. 作用域
- 目录：`editors/`（当前重点：`editors/vscode/`）。
- 目标：提升编辑器开发体验，不与自举编译链路文档混写。

## 2. 检查约束（Editors 专用）
- 每次修改 `editors/**` 后必须运行：
  - `npm --prefix editors/vscode run check:all`
- 本地插件开发宿主统一入口：
  - `npm --prefix editors/vscode run dev:ide`
- Editors 流程不要求运行：
  - `./scripts/run_checks.sh`
  - `./scripts/run_lency_checks.sh`

## 3. 当前实现状态
- 扩展入口模块化：`src/core/* + src/providers/* + src/extension.ts`。
- 模式可视化：状态栏 `Lency: LSP/Fallback`。
- LSP 路径：支持 `lency.serverPath`（含 `${workspaceFolder}`）。
- 配置热更新：修改 `lency.serverPath` 后自动重连并切换模式。
- 2026-04-03：`.lcy` 文件图标已改为走 VSCode 语言贡献里的原生 `languages[].icon`，不再额外提供 `Lency File Icons` 独立主题去抢用户现有 file icon theme。
- 2026-04-03：VSCode 扩展打包/发布工作流统一放在仓库根目录 `.github/workflows/editor-release.yml`；Dependabot 也统一放在根目录 `.github/dependabot.yml`，不要在 `editors/` 里摆第二套假 `.github`。
- 2026-04-03：VSCode 扩展已补发布用 `README.md`、`.vscodeignore` 与 `media/icon.png`；发布页图标与包内容收敛到可发布状态，不再是开发目录裸奔。
- 2026-04-03：扩展发布 tag 口径已收口为 `evX.Y.Z`；主项目继续使用 `vX.Y.Z`，不要再把插件 tag 写成长串 `vscode-v...`。
- 2026-04-03：扩展 release workflow 现已在 CI 内根据 `evX.Y.Z` 自动同步 `editors/vscode/package.json` 的 `version` 后再打包；tag 是唯一真值，不再要求手工先改版本号。

## 4. 已知边界

无待处理的 FIXME / TODO。

