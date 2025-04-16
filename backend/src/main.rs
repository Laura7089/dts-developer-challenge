//! Backend application to serve [`TodoTask`] objects over HTTP.

#![deny(clippy::pedantic)]
#![deny(missing_docs)]

use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response,
    routing::get,
};
use clap::Parser;
use dts_developer_challenge::{TodoStatus, TodoTask};
use sqlx::postgres::{PgConnectOptions, PgPool};
use tracing::{debug, info};
use uuid::Uuid;

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
    #[clap(long)]
    db_name: Option<String>,
    /// Username to access the Postgres database as.
    #[clap(long, default_value = "postgres")]
    db_user: String,
    /// Password for write access to the database in Postgres.
    ///
    /// Connects without password by default.
    #[clap(long)]
    db_password_file: Option<PathBuf>,
    /// Enable verbose logging.
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Skip running the database migrations on startup.
    #[clap(long, default_value_t = false)]
    skip_migrations: bool,
}

#[tokio::main]
#[tracing::instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("starting application");

    let opts = Opt::parse();

    // assemble connection options
    let mut db_options = PgConnectOptions::new()
        .host(&opts.db_host)
        .port(opts.db_port)
        .username(&opts.db_user);
    if let Some(db_name) = opts.db_name {
        db_options = db_options.database(&db_name);
    }
    if let Some(path) = opts.db_password_file {
        debug!(
            "read database password from {}",
            path.as_os_str().to_string_lossy()
        );
        let password = std::fs::read_to_string(path).expect("failed to read DB password file");
        db_options = db_options.password(password.trim());
    }

    // connect to the database
    let db_pool = PgPool::connect_with(db_options)
        .await
        .expect("failed to connect to database");
    info!(
        "database connection pool established at {}:{}",
        opts.db_host, opts.db_port
    );

    // run database migrations, if enabled
    if opts.skip_migrations {
        info!("skipping database migrations");
    } else {
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("migrations run failed");
        info!("database migrations complete");
    }

    let app = Router::new()
        .route("/task/{task_id}", get(get_task).post(post_task))
        .with_state(Arc::new(db_pool));

    let listener = tokio::net::TcpListener::bind(opts.service_address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tracing::instrument]
async fn get_task(
    State(pool): State<Arc<PgPool>>,
    Path(task_id): Path<Uuid>,
) -> Result<response::Json<TodoTask>, StatusCode> {
    let task = sqlx::query_as("SELECT * FROM tasks WHERE id == ?1")
        .bind(task_id)
        .fetch_one(Arc::as_ref(&pool))
        .await
        .expect("SQL query failure");

    Ok(response::Json(task))
}

#[tracing::instrument]
async fn post_task(
    State(pool): State<Arc<PgPool>>,
    Path(task_id): Path<Uuid>,
    Json(task): Json<TodoTask>,
) -> Result<(), StatusCode> {
    sqlx::query("INSERT INTO tasks (id, title, description, status, due) (?1, ?2, ?3, ?4, ?5)")
        .bind(&task_id)
        .bind(task.title())
        .bind(task.description())
        // TODO: implement sqlx::Encode for TodoStatus
        .bind(match task.status {
            TodoStatus::NotStarted => "not_started",
            TodoStatus::InProgress => "in_progress",
            TodoStatus::Complete => "complete",
            TodoStatus::Cancelled => "cancelled",
            TodoStatus::Blocked => "blocked",
        })
        .bind(task.due())
        .execute(Arc::as_ref(&pool))
        .await
        // TODO: handle error better
        .expect("SQL query failure");

    Ok(())
}
