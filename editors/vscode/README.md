# Lency Professional Support

Professional support for the Lency language in Visual Studio Code.

## Features

- Syntax highlighting for `.lcy`
- Snippets for common language constructs
- Local fallback diagnostics for common single-file issues
- Optional LSP mode through `lency_ls`
- File icon theme for `.lcy`

## Requirements

For full semantic features, build `lency_ls` from the monorepo root:

```powershell
cargo build -p lency_ls
```

If the server is not auto-detected, configure `lency.serverPath` in VS Code.

## Development

From the monorepo root:

```powershell
npm --prefix editors/vscode run check:all
npm --prefix editors/vscode run dev:ide
```

## Packaging

From `editors/vscode`:

```powershell
npm run package:vsix
```

## Release

- Main project release tags use `vX.Y.Z`
- VSCode extension release tags use `evX.Y.Z`
- VSCode extension packaging syncs `package.json` version from the `ev` tag inside CI before building the `.vsix`

Do not mix them. The extension is packaged by the root workflow:

- `.github/workflows/editor-release.yml`
