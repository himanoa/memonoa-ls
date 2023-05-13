use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ast::{MemonoaLine, TokenizeContext, MemonoaWord};
use tokio::fs::{read_dir, read_to_string};
use tokio::main;
use tower_lsp::async_trait;
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing_subscriber::prelude::*;
use wakachigaki::tiny_segmenter_wakachigaki::TinySegmentWakachigaki;

pub mod ast;
pub mod range;
mod wakachigaki;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<String, PathBuf>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Backend {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
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
        let mut entries = read_dir(".")
            .await
            .map_err(|_| Error::new(ErrorCode::ServerError(1)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| Error::new(ErrorCode::ServerError(1)))?
        {
            self.client
                .log_message(MessageType::INFO, format!("{:?}", entry))
                .await;
            let mut documents = self
                .documents
                .lock()
                .map_err(|_| Error::new(ErrorCode::ServerError(1)))?;
            if let Some(file_name) = entry.path().file_stem().and_then(|s| s.to_str()) {
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.client
            .log_message(MessageType::INFO, format!("{:?}", params))
            .await;
        let file_path = params.text_document_position_params.text_document.uri.path();
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let client = self.client.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                client.log_message(MessageType::INFO, msg).await;
            }
        });
        tx.send("get file_path".to_string()).await;
        self.client
            .log_message(MessageType::INFO, "get file_path")
            .await;
        let file_body = match read_to_string(file_path).await {
            Ok(body) => body,
            Err(_) => { return Ok(None) }
        };
        let cursor_position = params.text_document_position_params.position.character;
        let line_with_cursor = match file_body.lines().nth(params.text_document_position_params.position.line as usize) {
            Some(line) => line,
            None => { return Ok(None) }
        };
        let documents = match self.documents.try_lock() {
            Ok(documents) => documents,
            Err(_) => { return Ok(None) }
        };
        let tokenize_context = TokenizeContext::new(TinySegmentWakachigaki::new(), &documents);
        let words = MemonoaLine::tokenize(tokenize_context, line_with_cursor.to_string());
        tx.try_send(format!("{:?}", words.clone()));
        tx.try_send(format!("position {}", cursor_position));
        tx.try_send(format!("document {:?}", documents));
        let go_to_file_path = match words.0.into_iter().find(|w| w.is_selected(cursor_position as usize)) {
            Some(MemonoaWord::Link { path, .. }) => path,
            _ => { return Ok(None) }
        };
        tx.try_send(format!("{:?}", go_to_file_path.clone()));
        let u = Url::try_from(go_to_file_path);
        tx.try_send(format!("{:?}", u.clone()));
        Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
            u, Range::new(Position::new(0, 0), Position::new(0,0))
        ))))
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
