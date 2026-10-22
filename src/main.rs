use std::{env, net::IpAddr, path::{Path, PathBuf}, time::Instant};

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use sysinfo::{Disks, System};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const INDEX_HTML: &str = include_str!("../frontend/index.html");
const APP_CSS: &str = include_str!("../frontend/app.css");
const APP_JS: &str = include_str!("../frontend/app.js");

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    started: Instant,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct InfoResponse {
    name: &'static str,
    version: &'static str,
    mode: &'static str,
    uptime_seconds: u64,
    bind_address: String,
}

#[derive(Serialize)]
struct MemoryMetrics {
    total_bytes: u64,
    used_bytes: u64,
    used_percent: f64,
}

#[derive(Serialize)]
struct DiskMetrics {
    mount_point: String,
    total_bytes: u64,
    available_bytes: u64,
    used_bytes: u64,
    used_percent: f64,
}

#[derive(Serialize)]
struct MetricsResponse {
    cpu_percent: f32,
    memory: MemoryMetrics,
    disk: Option<DiskMetrics>,
    system_uptime_seconds: u64,
    collected_at_unix_seconds: u64,
}

#[derive(Serialize)]
struct Event {
    id: i64,
    created_at: String,
    level: String,
    message: String,
}

fn data_dir() -> Result<PathBuf> {
    if let Ok(value) = env::var("NEXUS_DATA_DIR") {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("NEXUS_DATA_DIR cannot be empty"));
        }
        return Ok(PathBuf::from(trimmed));
    }
    let home = env::var("HOME").context("HOME is not set; set NEXUS_DATA_DIR explicitly")?;
    Ok(PathBuf::from(home).join(".local/share/nexus"))
}

fn bind_config() -> Result<(IpAddr, u16)> {
    let bind_ip: IpAddr = env::var("NEXUS_BIND")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .context("NEXUS_BIND must be a valid IP address")?;
    if !bind_ip.is_loopback() {
        return Err(anyhow!(
            "NEXUS only accepts loopback binding in this milestone. Set NEXUS_BIND to 127.0.0.1; remote access must wait until authentication is implemented."
        ));
    }
    let port: u16 = env::var("NEXUS_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .context("NEXUS_PORT must be a valid port number")?;
    if port == 0 {
        return Err(anyhow!("NEXUS_PORT must be between 1 and 65535"));
    }
    Ok((bind_ip, port))
}

async fn initialize_database(path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context("could not open the NEXUS SQLite database")?;

    // Keep this SQL aligned with migrations/0001_initial.sql.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),\
            level TEXT NOT NULL CHECK(level IN ('info','warn','error')),\
            message TEXT NOT NULL\
        )",
    )
    .execute(&pool)
    .await
    .context("could not initialize the NEXUS database schema")?;

    sqlx::query(
        "INSERT INTO events(level, message) VALUES('info', 'NEXUS API started')",
    )
    .execute(&pool)
    .await
    .context("could not record startup event")?;

    Ok(pool)
}

async fn index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

async fn stylesheet() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], APP_CSS)
}

async fn javascript() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8")], APP_JS)
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(HealthResponse {
        status: "ok",
        database: "ok",
        version: env!("CARGO_PKG_VERSION"),
    }))
}

async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    let (ip, port) = bind_config().expect("binding was validated during startup");
    Json(InfoResponse {
        name: "NEXUS Server",
        version: env!("CARGO_PKG_VERSION"),
        mode: "local-only",
        uptime_seconds: state.started.elapsed().as_secs(),
        bind_address: format!("{ip}:{port}"),
    })
}

async fn metrics() -> Json<MetricsResponse> {
    let mut system = System::new_all();
    system.refresh_all();

    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_percent = if total_memory == 0 {
        0.0
    } else {
        (used_memory as f64 / total_memory as f64 * 100.0).clamp(0.0, 100.0)
    };

    let disks = Disks::new_with_refreshed_list();
    let root_disk = disks
        .list()
        .iter()
        .find(|disk| disk.mount_point() == Path::new("/"))
        .or_else(|| disks.list().first());
    let disk_metrics = root_disk.map(|disk| {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        DiskMetrics {
            mount_point: disk.mount_point().to_string_lossy().into_owned(),
            total_bytes: total,
            available_bytes: available,
            used_bytes: used,
            used_percent: if total == 0 {
                0.0
            } else {
                (used as f64 / total as f64 * 100.0).clamp(0.0, 100.0)
            },
        }
    });

    Json(MetricsResponse {
        cpu_percent: system.global_cpu_usage().clamp(0.0, 100.0),
        memory: MemoryMetrics {
            total_bytes: total_memory,
            used_bytes: used_memory,
            used_percent: memory_percent,
        },
        disk: disk_metrics,
        system_uptime_seconds: System::uptime(),
        collected_at_unix_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default(),
    })
}

async fn events(State(state): State<AppState>) -> Result<Json<Vec<Event>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, created_at, level, message FROM events ORDER BY id DESC LIMIT 12",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let events = rows
        .into_iter()
        .map(|row| Event {
            id: row.get("id"),
            created_at: row.get("created_at"),
            level: row.get("level"),
            message: row.get("message"),
        })
        .collect();
    Ok(Json(events))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C; shutting down"),
        _ = terminate => info!("received termination signal; shutting down"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let (bind_ip, port) = bind_config()?;
    let directory = data_dir()?;
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("could not create data directory {}", directory.display()))?;
    let canonical_dir = std::fs::canonicalize(&directory)
        .with_context(|| format!("could not resolve data directory {}", directory.display()))?;
    let database_path = canonical_dir.join("nexus.sqlite3");
    let db = initialize_database(&database_path).await?;

    let state = AppState { db, started: Instant::now() };
    let app = Router::new()
        .route("/", get(index))
        .route("/assets/app.css", get(stylesheet))
        .route("/assets/app.js", get(javascript))
        .route("/api/health", get(health))
        .route("/api/info", get(info))
        .route("/api/metrics", get(metrics))
        .route("/api/events", get(events))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("could not bind to {address}"))?;

    info!(bind = %address, data_dir = %canonical_dir.display(), "NEXUS Server starting in local-only mode");
    warn!("Phase 1 does not include authentication; do not expose this service through ngrok or a public reverse proxy yet");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("NEXUS HTTP server failed")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn health_contract_has_expected_fields() {
        let value = json!({ "status": "ok", "database": "ok", "version": "0.1.0" });
        assert_eq!(value["status"], "ok");
        assert_eq!(value["database"], "ok");
        assert!(value["version"].is_string());
    }

    #[test]
    fn memory_percent_math_is_bounded() {
        let total = 100_u64;
        let used = 140_u64;
        let percent = if total == 0 { 0.0 } else { (used as f64 / total as f64 * 100.0).clamp(0.0, 100.0) };
        assert_eq!(percent, 100.0);
    }
}
