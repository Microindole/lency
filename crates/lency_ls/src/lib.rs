use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        eprintln!("[LencyLS] Received initialize request");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        eprintln!("[LencyLS] Server initialized and ready");
        self.client
            .log_message(MessageType::INFO, "Lency Language Server initialized!")
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params.position;
        eprintln!(
            "[LencyLS] Hover request at line {}, col {}",
            pos.line, pos.character
        );

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "✨ 正在由 Lency Rust LSP 提供实时语义支持".to_string(),
            )),
            range: None,
        }))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
