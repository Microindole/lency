# Lency 编辑器支持

`editors/` 包含 Lency 的编辑器集成，当前主要是 VS Code 扩展。

仓库当前的开发主线是 `lencyc/` 自举编译器。编辑器只消费已稳定的语言能力，不在扩展端定义平行语义。

## 文档

- [编辑器文档导航](./docs/README.md)
- [设计与边界](./docs/design.md)
- [当前实现状态](./docs/development/status.md)
- [VS Code 扩展用户与发布说明](./vscode/README.md)

## 开发入口

```powershell
npm --prefix editors/vscode run check:all
npm --prefix editors/vscode run dev:ide
```

源码位于 `editors/vscode/src/`；语法高亮和代码片段分别位于 `editors/vscode/syntaxes/` 和 `editors/vscode/snippets/`。
