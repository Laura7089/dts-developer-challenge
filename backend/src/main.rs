//! Backend application to serve [`TodoTask`] objects over HTTP.

#![deny(clippy::pedantic)]
#![deny(missing_docs)]

mod cli;
mod tasks;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use clap::Parser;
use sqlx::postgres::PgPool;
use tracing::{debug, error, info};
use uuid::Uuid;

use tasks::{TodoTask, TodoTaskUnchecked};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    // parse CLI options
    let opts = cli::Opt::parse();

    // initialise logging
    tracing_subscriber::fmt().init();

    info!("starting application");

    // connect to the database
    let db_pool = PgPool::connect_with(opts.db_options())
        .await
        .expect("failed to connect to database");
    info!("database connection pool established",);

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
