import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

export interface LspStartResult {
    client: LanguageClient | undefined;
    started: boolean;
}

export function resolveServerPath(context: vscode.ExtensionContext): string | undefined {
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    const configured = vscode.workspace.getConfiguration('lency').get<string>('serverPath');
    const executableName = process.platform === 'win32' ? 'lency_ls.exe' : 'lency_ls';

    if (configured && configured.trim().length > 0) {
        const trimmed = configured.trim();
        const replaced = workspaceRoot
            ? trimmed.replace('${workspaceFolder}', workspaceRoot)
            : trimmed;
        const candidate = path.isAbsolute(replaced)
            ? replaced
            : workspaceRoot
                ? path.resolve(workspaceRoot, replaced)
                : path.resolve(replaced);
        if (fs.existsSync(candidate)) {
            return candidate;
        }
        // FIXME: 当前仅在启动时检查 serverPath 是否存在，未监听配置变更自动重连。
        void vscode.window.showWarningMessage(`lency.serverPath 无效: ${candidate}`);
    }

    const candidates = [
        context.asAbsolutePath(`../../target/debug/${executableName}`),
        context.asAbsolutePath(`../../target/release/${executableName}`),
        workspaceRoot ? path.join(workspaceRoot, 'target/debug', executableName) : undefined,
        workspaceRoot ? path.join(workspaceRoot, 'target/release', executableName) : undefined
    ].filter((value): value is string => typeof value === 'string');

    return candidates.find(candidate => fs.existsSync(candidate));
}

export function startLanguageClient(
    context: vscode.ExtensionContext
): LspStartResult {
    const serverPath = resolveServerPath(context);
    if (!serverPath) {
        return { client: undefined, started: false };
    }

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: 'lency', scheme: 'file' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lcy')
        }
    };

    const client = new LanguageClient(
        'lencyLanguageServer',
        'Lency Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
    return { client, started: true };
}
