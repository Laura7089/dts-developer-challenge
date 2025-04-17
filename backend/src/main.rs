//! Backend application to serve [`TodoTask`] objects over HTTP.

#![deny(clippy::pedantic)]
#![deny(missing_docs)]

use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use clap::Parser;
use sqlx::postgres::{PgConnectOptions, PgPool};
use tracing::{debug, error, info};
use uuid::Uuid;

use dts_developer_challenge::{TodoTask, TodoTaskUnchecked};

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
    /// Skip running the database migrations on startup.
    #[clap(long, default_value_t = false)]
    skip_migrations: bool,
}

#[tokio::main]
#[tracing::instrument]
async fn main() {
    // parse CLI options
    let opts = Opt::parse();

    // initialise logging
    tracing_subscriber::fmt().init();

    info!("starting application");

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
        host = opts.db_host,
        port = opts.db_port,
        "database connection pool established",
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
        .route("/task/{task_id}", get(get_task))
        .route("/task", post(post_task))
        .with_state(Arc::new(db_pool));

    let listener = tokio::net::TcpListener::bind(opts.service_address)
        .await
        .expect("failed to bind listen address");
    axum::serve(listener, app)
        .await
        .expect("application serve failure");
}

#[tracing::instrument]
async fn get_task(
    State(pool): State<Arc<PgPool>>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TodoTask>, StatusCode> {
    let query = sqlx::query_as(
        r#"SELECT title, description, status as "status: TodoStatus", due
        FROM tasks
        WHERE id = $1"#,
    )
    .bind(task_id);

    match query.fetch_one(Arc::as_ref(&pool)).await {
        Ok(task) => Ok(Json(task)),
        // if the database returned no row, then the ID doesn't exist
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!(
                task_id = format!("{task_id}"),
                error = format!("{e}"),
                "database error trying to get task"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tracing::instrument]
async fn post_task(
    State(pool): State<Arc<PgPool>>,
    Json(task): Json<TodoTaskUnchecked>,
) -> Result<String, StatusCode> {
    // validate the task
    let task = match TodoTask::try_from(task) {
        Ok(t) => t,
        Err(e) => {
            debug!(error = format!("{e}"), "malformed task received");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let task_id = Uuid::new_v4();
    let status = task.status;
    let query = sqlx::query!(
        "INSERT INTO tasks (id, title, description, status, due)
        VALUES ($1, $2, $3, $4, $5);",
        task_id,
        task.title(),
        task.description(),
        status as _,
        task.due(),
    );

    match query.execute(Arc::as_ref(&pool)).await {
        Ok(_) => Ok(format!("{task_id}")),
        Err(e) => {
            error!(
                error = format!("{e}"),
                "database error trying to create task"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
