use std::{collections::BTreeMap, time::Duration};

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
    let (docs_root, explicit_config, bind_address, rust_log) = parse_args(std::env::args())?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| rust_log.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let file_reader = FileReader::new(docs_root.to_string_lossy().to_string())?;
    let cfg = Config::load(explicit_config.as_deref())?;
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
    let tcp_listener = tokio::net::TcpListener::bind(&bind_address).await?;
    info!(
        "MCP server started on {}, docs_root: {}, RUST_LOG: {}",
        bind_address,
        file_reader.docs_root(),
        rust_log
    );
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(setup_graceful_shutdown())
        .await;
    Ok(())
}

fn parse_args(
    mut args: impl Iterator<Item = String>,
) -> anyhow::Result<(
    std::path::PathBuf,
    Option<std::path::PathBuf>,
    String,
    String,
)> {
    let _exe = args.next();

    let mut docs_root: Option<std::path::PathBuf> = None;
    let mut config: Option<std::path::PathBuf> = None;
    let mut bind_address: Option<String> = None;
    let mut rust_log: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--docs-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--docs-root requires a value"))?;
                docs_root = Some(std::path::PathBuf::from(value));
            }
            "--config" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--config requires a value"))?;
                config = Some(std::path::PathBuf::from(value));
            }
            "--bind-address" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--bind-address requires a value"))?;
                bind_address = Some(value);
            }
            "--rust-log" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--rust-log requires a value"))?;
                rust_log = Some(value);
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown argument '{}'. Expected --docs-root <path> [--config <path>] [--bind-address <addr>] [--rust-log <level>]",
                    arg
                ));
            }
        }
    }

    let docs_root = docs_root.ok_or_else(|| anyhow::anyhow!("--docs-root is required"))?;
    let bind_address = bind_address.unwrap_or_else(|| "127.0.0.1:8010".to_string());
    let rust_log = rust_log.unwrap_or_else(|| "info".to_string());
    Ok((docs_root, config, bind_address, rust_log))
}
