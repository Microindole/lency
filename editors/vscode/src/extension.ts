import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

/**
 * Lency 语言内置文档定义
 */
const BUILTIN_DOCS: { [key: string]: string } = {
    'print': '`void print(string msg)`\n\n将字符串打印到标准输出。',
    'read_file': '`string! read_file(string path)`\n\n读取文件全部内容。返回可空的结果类型。',
    'write_file': '`void! write_file(string path, string content)`\n\n将内容写入指定文件。',
    'len': '`int len(string s)`\n\n返回字符串的长度。',
    'trim': '`string trim(string s)`\n\n去除字符串首尾的空白字符。',
    'split': '`vec<string> split(string s, string sep)`\n\n使用分隔符拆分字符串。',
    'join': '`string join(vec<string> parts, string sep)`\n\n使用分隔符连接字符串数组。',
    'substr': '`string substr(string s, int start, int length)`\n\n获取子字符串。',
    'var': '声明一个变量。支持类型推导。\n\n例：`var x = 10`',
    'const': '声明一个常量。',
    'struct': '定义一个结构体。',
    'trait': '定义一个接口特征。',
    'impl': '为类型实现方法或特征。',
    'this': '指向当前实例的引用。',
    'null': '空值字面量。'
};

export function activate(context: vscode.ExtensionContext) {
    console.log('Lency Professional V6+ extension is now active!');

    // LSP Server 设置
    const serverPath = context.asAbsolutePath('../../target/debug/lency_ls');
    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'lency' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lcy')
        }
    };

    client = new LanguageClient(
        'lencyLanguageServer',
        'Lency Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    const selector: vscode.DocumentSelector = { language: 'lency' };

    // 1. 注册 Document Symbol Provider (大纲视图)
    context.subscriptions.push(
        vscode.languages.registerDocumentSymbolProvider(selector, new LencyDocumentSymbolProvider())
    );

    // 2. 注册 Hover Provider (悬停文档)
    context.subscriptions.push(
        vscode.languages.registerHoverProvider(selector, new LencyHoverProvider())
    );

    // 3. 注册 Document Highlight Provider (引用高亮)
    context.subscriptions.push(
        vscode.languages.registerDocumentHighlightProvider(selector, new LencyDocumentHighlightProvider())
    );

    // 4. 注册 Completion Item Provider (智能补全)
    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider(selector, new LencyCompletionProvider(), '.', '<')
    );

    // 5. 注册 Signature Help Provider (方法签名提示)
    context.subscriptions.push(
        vscode.languages.registerSignatureHelpProvider(selector, new LencySignatureHelpProvider(), '(', ',')
    );

    // 6. 注册 Document Formatting Edit Provider (基础格式化)
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider(selector, new LencyFormattingProvider())
    );

    // 7. 注册 Definition Provider (跳转到定义)
    context.subscriptions.push(
        vscode.languages.registerDefinitionProvider(selector, new LencyDefinitionProvider())
    );

    // 8. 注册 Rename Provider (重命名)
    context.subscriptions.push(
        vscode.languages.registerRenameProvider(selector, new LencyRenameProvider())
    );
}

class LencyDefinitionProvider implements vscode.DefinitionProvider {
    provideDefinition(document: vscode.TextDocument, position: vscode.Position): vscode.ProviderResult<vscode.Definition> {
        const range = document.getWordRangeAtPosition(position);
        if (!range) return null;
        const word = document.getText(range);

        // 搜索当前文件中的定义（简单正则匹配）
        const text = document.getText();
        const regexes = [
            new RegExp(`\\b(struct|trait|enum|impl)\\s+${word}\\b`, 'g'),
            new RegExp(`\\b(void|int|float|bool|string|[A-Z][a-zA-Z0-9_]*)\\b\\s+${word}\\s*\\(`, 'g')
        ];

        for (const regex of regexes) {
            let match;
            while ((match = regex.exec(text)) !== null) {
                const targetPos = document.positionAt(match.index + match[0].indexOf(word));
                return new vscode.Location(document.uri, targetPos);
            }
        }
        return null;
    }
}

class LencyFormattingProvider implements vscode.DocumentFormattingEditProvider {
    provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
        const edits: vscode.TextEdit[] = [];
        let indentLevel = 0;
        const tabSize = 4;

        for (let i = 0; i < document.lineCount; i++) {
            const line = document.lineAt(i);
            let text = line.text.trim();

            if (text.length === 0) continue;

            if (text.startsWith('}')) {
                indentLevel = Math.max(0, indentLevel - 1);
            }

            const expectedIndent = ' '.repeat(indentLevel * tabSize);
            if (line.firstNonWhitespaceCharacterIndex !== indentLevel * tabSize || line.text !== expectedIndent + text) {
                edits.push(vscode.TextEdit.replace(line.range, expectedIndent + text));
            }

            if (text.endsWith('{')) {
                indentLevel++;
            }
        }
        return edits;
    }
}

class LencyHoverProvider implements vscode.HoverProvider {
    provideHover(document: vscode.TextDocument, position: vscode.Position): vscode.Hover | null {
        const range = document.getWordRangeAtPosition(position);
        if (!range) return null;
        const word = document.getText(range);

        if (BUILTIN_DOCS[word]) {
            return new vscode.Hover(new vscode.MarkdownString(BUILTIN_DOCS[word]));
        }
        return null;
    }
}

class LencyDocumentHighlightProvider implements vscode.DocumentHighlightProvider {
    provideDocumentHighlights(document: vscode.TextDocument, position: vscode.Position): vscode.DocumentHighlight[] {
        const range = document.getWordRangeAtPosition(position);
        if (!range) return [];

        const word = document.getText(range);
        const highlights: vscode.DocumentHighlight[] = [];
        const text = document.getText();
        const regex = new RegExp(`\\b${word}\\b`, 'g');

        let match;
        while ((match = regex.exec(text)) !== null) {
            const startPos = document.positionAt(match.index);
            const endPos = document.positionAt(match.index + word.length);
            highlights.push(new vscode.DocumentHighlight(
                new vscode.Range(startPos, endPos),
                vscode.DocumentHighlightKind.Text
            ));
        }

        return highlights;
    }
}

class LencyDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
    provideDocumentSymbols(document: vscode.TextDocument): vscode.DocumentSymbol[] {
        const symbols: vscode.DocumentSymbol[] = [];
        const text = document.getText();

        const typeRegex = /^(?:struct|trait|enum)\s+([a-zA-Z_][a-zA-Z0-9_]*)/gm;
        let match;
        while ((match = typeRegex.exec(text)) !== null) {
            const name = match[1];
            const line = document.lineAt(document.positionAt(match.index).line);
            symbols.push(new vscode.DocumentSymbol(
                name,
                '',
                vscode.SymbolKind.Struct,
                line.range,
                line.range
            ));
        }

        const implRegex = /^impl(?:<.*>)?\s+([a-zA-Z_][a-zA-Z0-9_]*)/gm;
        while ((match = implRegex.exec(text)) !== null) {
            const name = `impl ${match[1]}`;
            const line = document.lineAt(document.positionAt(match.index).line);
            symbols.push(new vscode.DocumentSymbol(
                name,
                '',
                vscode.SymbolKind.Interface,
                line.range,
                line.range
            ));
        }

        const funcRegex = /^(?:\b(?:void|int|float|bool|string|[A-Z][a-zA-Z0-9_]*)\b)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/gm;
        while ((match = funcRegex.exec(text)) !== null) {
            const name = match[1];
            const line = document.lineAt(document.positionAt(match.index).line);
            symbols.push(new vscode.DocumentSymbol(
                name,
                '',
                vscode.SymbolKind.Function,
                line.range,
                line.range
            ));
        }

        return symbols;
    }
}

class LencyCompletionProvider implements vscode.CompletionItemProvider {
    provideCompletionItems(document: vscode.TextDocument, position: vscode.Position): vscode.CompletionItem[] {
        const completions: vscode.CompletionItem[] = [];

        ['var', 'const', 'struct', 'impl', 'trait', 'enum', 'if', 'else', 'while', 'for', 'return', 'import', 'match'].forEach(k => {
            const item = new vscode.CompletionItem(k, vscode.CompletionItemKind.Keyword);
            completions.push(item);
        });

        Object.keys(BUILTIN_DOCS).forEach(func => {
            if (func.length > 3) {
                const item = new vscode.CompletionItem(func, vscode.CompletionItemKind.Function);
                item.detail = 'Lency Built-in';
                item.documentation = new vscode.MarkdownString(BUILTIN_DOCS[func]);
                completions.push(item);
            }
        });

        return completions;
    }
}

class LencySignatureHelpProvider implements vscode.SignatureHelpProvider {
    provideSignatureHelp(document: vscode.TextDocument, position: vscode.Position): vscode.SignatureHelp | null {
        const textBefore = document.getText(new vscode.Range(new vscode.Position(position.line, 0), position));
        const lastOpenParen = textBefore.lastIndexOf('(');
        if (lastOpenParen === -1) return null;

        const functionNameMatch = textBefore.substring(0, lastOpenParen).match(/([a-zA-Z_][a-zA-Z0-9_]*)\s*$/);
        if (!functionNameMatch) return null;

        const name = functionNameMatch[1];
        if (BUILTIN_DOCS[name]) {
            const help = new vscode.SignatureHelp();
            const sigDoc = new vscode.MarkdownString(BUILTIN_DOCS[name]);
            const signature = new vscode.SignatureInformation(name, sigDoc);

            const paramText = textBefore.substring(lastOpenParen + 1);
            const activeParam = (paramText.match(/,/g) || []).length;

            if (name === 'read_file' || name === 'print' || name === 'len') {
                signature.parameters = [new vscode.ParameterInformation('arg')];
            } else if (name === 'split' || name === 'join' || name === 'write_file') {
                signature.parameters = [
                    new vscode.ParameterInformation('arg1'),
                    new vscode.ParameterInformation('arg2')
                ];
            } else if (name === 'substr') {
                signature.parameters = [
                    new vscode.ParameterInformation('s'),
                    new vscode.ParameterInformation('start'),
                    new vscode.ParameterInformation('length')
                ];
            }

            help.signatures = [signature];
            help.activeSignature = 0;
            help.activeParameter = Math.min(activeParam, (signature.parameters?.length || 1) - 1);
            return help;
        }

        return null;
    }
}

class LencyRenameProvider implements vscode.RenameProvider {
    provideRenameEdits(document: vscode.TextDocument, position: vscode.Position, newName: string): vscode.WorkspaceEdit {
        const range = document.getWordRangeAtPosition(position);
        if (!range) return new vscode.WorkspaceEdit();

        const word = document.getText(range);
        const workspaceEdit = new vscode.WorkspaceEdit();
        const text = document.getText();

        const regex = new RegExp(`\\b${word}\\b`, 'g');
        let match;
        while ((match = regex.exec(text)) !== null) {
            const startPos = document.positionAt(match.index);
            const endPos = document.positionAt(match.index + word.length);
            workspaceEdit.replace(document.uri, new vscode.Range(startPos, endPos), newName);
        }

        return workspaceEdit;
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
