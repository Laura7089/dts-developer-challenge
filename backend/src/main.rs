//! Backend application to serve [`TodoTask`] objects over HTTP.

#![deny(clippy::pedantic)]
#![deny(missing_docs)]

use axum::Router;
use clap::Parser;

/// Command-line arguments of the application.
#[derive(Parser, Debug, Clone)]
struct Opt {
    /// Address at which to serve the application.
    #[clap(default_value = "0.0.0.0:8080")]
    service_address: String,
    /// Address to contact the Postgres server on.
    #[clap(long)]
    db_address: String,
    /// Name of the database to open in Postgres.
    #[clap(long, default_value = "tasks_db")]
    db_name: String,
    /// Enable verbose logging.
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts = Opt::parse();

    let app = Router::new();

    let listener = tokio::net::TcpListener::bind(opts.service_address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
