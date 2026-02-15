use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use clap::Parser;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing::{info, warn};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
mod config;
mod models;
mod server;
mod utils;
use config::Config;
use models::{DocumentKey, DocumentScanner, DocumentType, ResourceInfo};
use server::DocumentServer;

use crate::utils::file_reader::FileReader;

/// MCP server for architecture docs — in the Emperor's name, serve the docs.
#[derive(Parser, Debug)]
#[command(name = "arch-mcp-server")]
struct Cli {
    /// Root directory for documentation (required).
    #[arg(long, value_name = "PATH")]
    docs_root: PathBuf,

    /// Path to config file (arch-mcp.toml). Default: current dir.
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Address to bind (host:port).
    #[arg(long, value_name = "ADDR", default_value = "127.0.0.1:8010")]
    bind_address: String,

    /// RUST_LOG-style level when RUST_LOG env is unset.
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    rust_log: String,
}

impl Cli {
    fn docs_root(&self) -> &PathBuf {
        &self.docs_root
    }
    fn config(&self) -> Option<&PathBuf> {
        self.config.as_ref()
    }
    fn bind_address(&self) -> &str {
        &self.bind_address
    }
    fn rust_log(&self) -> &str {
        &self.rust_log
    }
}

#[allow(clippy::ignored_unit_patterns)]
async fn setup_graceful_shutdown() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received ctrl+c, shutting down gracefully...");
        }
        _ = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
                let mut sigint = signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");

                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM, shutting down gracefully...");
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT, shutting down gracefully...");
                    }
                }
            }
            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
                info!("Received shutdown signal, shutting down gracefully...");
            }
        } => {}
    }

    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        warn!("Graceful shutdown timeout reached, forcing exit...");
        std::process::exit(0);
    });
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| cli.rust_log().to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let file_reader = FileReader::new(cli.docs_root().to_string_lossy().to_string())?;
    let cfg = Config::load(cli.config().map(std::path::PathBuf::as_path))?;
    let mut resources: BTreeMap<DocumentKey, ResourceInfo> = BTreeMap::new();

    let scan_start = std::time::Instant::now();

    // Scan agreements
    let area_paths = cfg.agreements.clone();
    DocumentScanner::scan_documents(
        DocumentType::Agreements,
        area_paths,
        &file_reader,
        &mut resources,
    );

    for project in &cfg.projects {
        let diagram_exts = cfg.diagram_extensions.clone();
        let openapi_exts = cfg.openapi_extensions.clone();

        let mut scan_type =
            |document_type: DocumentType, targets: Vec<String>, exts: Vec<String>| {
                DocumentScanner::scan_documents_with_extensions(
                    document_type,
                    targets,
                    &exts,
                    &file_reader,
                    &mut resources,
                );
            };

        scan_type(
            DocumentType::C1Diagram(project.name.clone()),
            project.c4.c1.clone(),
            diagram_exts.clone(),
        );
        scan_type(
            DocumentType::C2Diagram(project.name.clone()),
            project.c4.c2.clone(),
            diagram_exts.clone(),
        );
        scan_type(
            DocumentType::C3Diagram(project.name.clone()),
            project.c4.c3.clone(),
            diagram_exts.clone(),
        );
        scan_type(
            DocumentType::C4Diagram(project.name.clone()),
            project.c4.services.clone(),
            diagram_exts.clone(),
        );
        scan_type(
            DocumentType::ErdDiagram(project.name.clone()),
            project.erd.clone(),
            diagram_exts.clone(),
        );
        scan_type(
            DocumentType::AdrDocument(project.name.clone()),
            project.adr.clone(),
            diagram_exts.clone(),
        );
        scan_type(
            DocumentType::OpenApiSpec(project.name.clone()),
            project.openapi.clone(),
            openapi_exts.clone(),
        );
    }

    let guide_exts = cfg.guide_extensions.clone();
    for guide in &cfg.guides {
        DocumentScanner::scan_documents_with_extensions(
            DocumentType::GuideDoc(guide.name.clone()),
            guide.paths.clone(),
            &guide_exts,
            &file_reader,
            &mut resources,
        );
    }

    let scan_duration = scan_start.elapsed();
    info!(
        "Scanned {} documents in {:?}",
        resources.len(),
        scan_duration
    );

    let server_file_reader = file_reader.clone();
    let service = StreamableHttpService::new(
        move || {
            Ok(DocumentServer::new_with_resources(
                server_file_reader.clone(),
                resources.clone(),
            ))
        },
        LocalSessionManager::default().into(),
        rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(cli.bind_address()).await?;
    info!(
        "MCP server started on {}, docs_root: {}, RUST_LOG: {}",
        cli.bind_address(),
        file_reader.docs_root(),
        cli.rust_log()
    );
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(setup_graceful_shutdown())
        .await;
    Ok(())
}
