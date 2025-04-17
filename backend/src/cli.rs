use clap::Parser;
use sqlx::postgres::PgConnectOptions;
use std::path::PathBuf;
use tracing::debug;

/// Command-line arguments of the application.
#[derive(Parser, Debug, Clone)]
pub(crate) struct Opt {
    /// Address at which to serve the application.
    #[clap(default_value = "0.0.0.0:8080")]
    pub service_address: String,
    /// Address to contact the Postgres server on.
    #[clap(long)]
    pub db_host: String,
    /// Port to contact the Postgres server on.
    #[clap(long, default_value_t = 5432)]
    pub db_port: u16,
    /// Name of the database to open in Postgres.
    #[clap(long)]
    pub db_name: Option<String>,
    /// Username to access the Postgres database as.
    #[clap(long, default_value = "postgres")]
    pub db_user: String,
    /// Password for write access to the database in Postgres.
    ///
    /// Connects without password by default.
    #[clap(long)]
    pub db_password_file: Option<PathBuf>,
    /// Skip running the database migrations on startup.
    #[clap(long, default_value_t = false)]
    pub skip_migrations: bool,
}

impl Opt {
    #[tracing::instrument]
    pub(crate) fn db_options(&self) -> PgConnectOptions {
        // assemble connection options
        let mut db_options = PgConnectOptions::new()
            .host(&self.db_host)
            .port(self.db_port)
            .username(&self.db_user);
        if let Some(db_name) = self.db_name.as_deref() {
            db_options = db_options.database(db_name);
        }
        if let Some(path) = self.db_password_file.as_deref() {
            debug!(
                "read database password from {}",
                path.as_os_str().to_string_lossy()
            );
            let password = std::fs::read_to_string(path).expect("failed to read DB password file");
            db_options = db_options.password(password.trim());
        }

        db_options
    }
}
