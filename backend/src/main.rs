//! Backend application to serve [`TodoTask`] objects over HTTP.

#![deny(clippy::pedantic)]
#![deny(missing_docs)]

use axum::Router;
use clap::Parser;
use sqlx::{
    Pool,
    postgres::{PgConnectOptions, Postgres},
};

/// Command-line arguments of the application.
#[derive(Parser, Debug, Clone)]
struct Opt {
    /// Address at which to serve the application.
    #[clap(default_value = "0.0.0.0:8080")]
    service_address: String,
    /// Address to contact the Postgres server on.
    #[clap(long)]
    db_host: String,
    /// Port to contact the Postgres server on.
    #[clap(long, default_value_t = 5432)]
    db_port: u16,
    /// Name of the database to open in Postgres.
    #[clap(long, default_value = "tasks_db")]
    db_name: String,
    /// Password for write access to the database in Postgres.
    #[clap(long)]
    db_password: Option<String>,
    /// Enable verbose logging.
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Skip running the database migrations on startup.
    #[clap(long, default_value_t = false)]
    skip_migrations: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts = Opt::parse();

    // connect to the database
    let mut db_options = PgConnectOptions::new()
        .database(&opts.db_name)
        .host(&opts.db_host)
        .port(opts.db_port);
    if let Some(db_pass) = opts.db_password {
        db_options = db_options.password(&db_pass);
    }
    let db_pool: Pool<Postgres> = Pool::connect_with(db_options)
        .await
        .expect("failed to connect to database");

    // run database migrations, if enabled
    if !opts.skip_migrations {
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("migrations run failed");
    }

    let app = Router::new();

    let listener = tokio::net::TcpListener::bind(opts.service_address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
