use lency_ls::Backend;
use serde_json::json;
use tower::Service;
use tower_lsp::LspService;

#[tokio::test]
async fn test_lsp_initialize() {
    let (mut service, _) = LspService::new(|client| Backend { client });

    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "processId": null,
            "rootUri": null,
            "workspaceFolders": null
        }
    });

    let request_str = serde_json::to_string(&init_request).unwrap();
    let request: tower_lsp::jsonrpc::Request = serde_json::from_str(&request_str).unwrap();

    let response: Option<tower_lsp::jsonrpc::Response> = service.call(request).await.unwrap();

    match response {
        Some(res) => {
            assert!(res.error().is_none());
            assert!(res.result().is_some());
            assert_eq!(res.id(), &tower_lsp::jsonrpc::Id::Number(1));
        }
        None => panic!("LSP returned None"),
    }
}
