use std::collections::HashMap;
use std::iter::Iterator;
use std::path::PathBuf;
use std::sync::{Mutex, Arc};

use tracing::{info, instrument};
use tracing_subscriber::prelude::*;
use tower_lsp::jsonrpc::{Result, Error};
use tower_lsp::jsonrpc::ErrorCode;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tower_lsp::async_trait;
use tokio::fs::read_dir;
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

#[async_trait]
impl LanguageServer for Backend {
    #[instrument]
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let mut dir = read_dir(".").await.map_err(|_| Error::new(ErrorCode::ServerError(1)))?;

        while let Ok(entry) = dir.next_entry().await {
            let mut documents = self.documents.lock().map_err(|_| Error::new(ErrorCode::ServerError(1)))?;
            let entry = match entry {
                Some(e) => e,
                None => { continue; }
            };
            if let Some(file_name) = entry.file_name().to_str() {
                documents.insert(file_name.to_string(), entry.path());
            }
        }

        Ok(InitializeResult::default())
    }

    #[instrument]
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    #[instrument]
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

    let (service, socket) = LspService::new( Backend::new);
    info!("Started memonoa language server");
    Server::new(stdin, stdout, socket).serve(service).await;
}
