use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, Arc};

use tokio::fs::read_dir;
use tracing_subscriber::prelude::*;
use tower_lsp::jsonrpc::{Result, Error, ErrorCode};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tower_lsp::async_trait;
use tokio::main;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<String, PathBuf>>>
}

impl Backend {
    fn new(client: Client) -> Self {
        Backend { client, documents: Arc::new(Mutex::new(HashMap::new())) }
    }
}

// TODO: Required refactor
//
// - Use tracing for logging.
//   - Implement tracing_layer used client#log_message
// - Split code. for each action.
//   - Write tests
#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.client.log_message(MessageType::INFO, "hello").await;
        self.client
            .log_message(MessageType::INFO, "Start initialize")
            .await;
        let mut entries = read_dir(".").await.map_err(|_| Error::new(ErrorCode::ServerError(1)))?;

        while let Some(entry) = entries.next_entry().await.map_err(|_| Error::new(ErrorCode::ServerError(1)))? {
            self.client.log_message(MessageType::INFO, format!("{:?}", entry)).await;
            let mut documents = self.documents.lock().map_err(|_| Error::new(ErrorCode::ServerError(1)))?;
            if let Some(file_name) = entry.file_name().to_str() {
                documents.insert(file_name.to_string(), entry.path());
            }
        }

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                position_encoding: None,
                text_document_sync: None,
                selection_range_provider: None,
                hover_provider: None,
                completion_provider: None,
                signature_help_provider: None,
                definition_provider: Some(OneOf::Left(true)),
                type_definition_provider: None,
                implementation_provider: None,
                references_provider: None,
                document_highlight_provider: None,
                document_symbol_provider: None,
                workspace_symbol_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                document_formatting_provider: None,
                document_range_formatting_provider: None,
                document_on_type_formatting_provider: None,
                rename_provider: None,
                document_link_provider: None,
                color_provider: None,
                folding_range_provider: None,
                declaration_provider: None,
                execute_command_provider: None,
                workspace: None,
                call_hierarchy_provider: None,
                semantic_tokens_provider: None,
                moniker_provider: None,
                inline_value_provider: None,
                inlay_hint_provider: None,
                linked_editing_range_provider: None,
                experimental: None,
            },
        })
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        self.client.log_message(MessageType::INFO, "hello").await;
        self.client.log_message(MessageType::INFO, format!("{:?}", params)).await;
        Ok(None)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (service, socket) = LspService::build(|client| Backend::new(client)).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
