use std::{
    collections::{HashMap, VecDeque},
    env,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    body::Body,
    extract::{ConnectInfo, Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow},
    Row, SqlitePool,
};
use sysinfo::{Disks, System};
use tokio::{net::TcpListener, sync::{Mutex, RwLock}};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const INDEX_HTML: &str = include_str!("../frontend/index.html");
const APP_CSS: &str = include_str!("../frontend/app.css");
const APP_JS: &str = include_str!("../frontend/app.js");
const SESSION_TTL_SECONDS: i64 = 12 * 60 * 60;
const AUTH_RATE_WINDOW_SECONDS: u64 = 15 * 60;
const AUTH_RATE_MAX_ATTEMPTS: usize = 10;
const MAX_BOTS_PER_USER: i64 = 10;
const MAX_BOTS_GLOBAL: i64 = 100;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    started: Instant,
    http: reqwest::Client,
    geo_cache: Arc<RwLock<HashMap<IpAddr, GeoCacheEntry>>>,
    auth_attempts: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
    telangana_only: bool,
    cookie_secure: bool,
}

#[derive(Clone)]
struct GeoCacheEntry {
    allowed: bool,
    expires_at: Instant,
}

#[derive(Deserialize)]
struct IpGeoResponse {
    country_code: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
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

#[derive(Deserialize)]
struct Credentials {
    email: String,
    password: String,
}

#[derive(Clone, Serialize)]
struct SafeUser {
    id: i64,
    email: String,
    role: String,
}

#[derive(Serialize)]
struct AuthStatus {
    initialized: bool,
    region_restriction: &'static str,
}

#[derive(Serialize)]
struct AuthPayload {
    user: SafeUser,
    csrf_token: String,
}

struct SessionData {
    user: SafeUser,
    csrf_token: String,
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

fn env_bool(name: &str, default: bool) -> Result<bool> {
    match env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(anyhow!("{name} must be true or false")),
        },
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}

fn bind_config() -> Result<(IpAddr, u16)> {
    let bind_ip: IpAddr = env::var("NEXUS_BIND")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .context("NEXUS_BIND must be a valid IP address")?;
    if !bind_ip.is_loopback() {
        return Err(anyhow!(
            "NEXUS Phase 2 only accepts loopback binding. Set NEXUS_BIND=127.0.0.1; do not expose it directly."
        ));
    }
    let port: u16 = env::var("NEXUS_PORT")
        .unwrap_or_else(|_| "3001".to_string())
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
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context("could not open the NEXUS SQLite database")?;

    for statement in [
        "CREATE TABLE IF NOT EXISTS events (id INTEGER PRIMARY KEY AUTOINCREMENT, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')), level TEXT NOT NULL CHECK(level IN ('info','warn','error')), message TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT NOT NULL UNIQUE COLLATE NOCASE, password_hash TEXT NOT NULL, role TEXT NOT NULL CHECK(role IN ('admin','user')), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')))",
        "CREATE TABLE IF NOT EXISTS sessions (token_hash TEXT PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, csrf_token TEXT NOT NULL, created_at INTEGER NOT NULL, expires_at INTEGER NOT NULL)",
        "CREATE INDEX IF NOT EXISTS sessions_expiry_idx ON sessions(expires_at)",
        "CREATE TABLE IF NOT EXISTS bots (id INTEGER PRIMARY KEY AUTOINCREMENT, owner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, name TEXT NOT NULL CHECK(length(name) BETWEEN 2 AND 48), description TEXT, runtime TEXT NOT NULL CHECK(runtime IN ('python','node')), runtime_version TEXT NOT NULL, entrypoint TEXT NOT NULL, cpu_limit_millicores INTEGER NOT NULL CHECK(cpu_limit_millicores BETWEEN 100 AND 2000), memory_limit_mb INTEGER NOT NULL CHECK(memory_limit_mb BETWEEN 128 AND 2048), desired_state TEXT NOT NULL DEFAULT 'stopped' CHECK(desired_state IN ('stopped','running')), observed_state TEXT NOT NULL DEFAULT 'stopped' CHECK(observed_state IN ('stopped','starting','running','stopping','failed','deploying','unavailable')), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')), updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')), last_activity_at TEXT)",
        "CREATE INDEX IF NOT EXISTS bots_owner_idx ON bots(owner_user_id, id DESC)",
        "CREATE TABLE IF NOT EXISTS bot_events (id INTEGER PRIMARY KEY AUTOINCREMENT, bot_id INTEGER NOT NULL REFERENCES bots(id) ON DELETE CASCADE, owner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, event_type TEXT NOT NULL, message TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')))",
        "CREATE INDEX IF NOT EXISTS bot_events_owner_bot_idx ON bot_events(owner_user_id, bot_id, id DESC)",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .context("could not initialize NEXUS database schema")?;
    }
    let now = unix_time();
    sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(now)
        .execute(&pool)
        .await
        .context("could not remove expired sessions")?;
    sqlx::query("INSERT INTO events(level, message) VALUES('info', 'NEXUS Phase 2 started')")
        .execute(&pool)
        .await
        .context("could not record startup event")?;
    Ok(pool)
}

fn unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn api_error(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(ApiError { error: message.into() })).into_response()
}

fn random_hex(byte_count: usize) -> String {
    let mut bytes = vec![0_u8; byte_count];
    let mut rng = OsRng;
    rng.fill_bytes(&mut bytes);
    let mut result = String::with_capacity(byte_count * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut result, "{byte:02x}");
    }
    result
}

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| anyhow!("password hashing failed: {error}"))
}

fn password_matches(password: &str, encoded_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(encoded_hash) else {
        return false;
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

fn normalize_email(email: &str) -> Option<String> {
    let value = email.trim().to_ascii_lowercase();
    if value.len() > 254 || value.contains(char::is_whitespace) {
        return None;
    }
    let (local, domain) = value.split_once('@')?;
    if local.is_empty() || domain.len() < 3 || !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return None;
    }
    Some(value)
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::COOKIE)?.to_str().ok()?;
    value.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        if name == "nexus_session" && value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn request_ip_from_headers(headers: &HeaderMap) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').filter_map(|part| part.trim().parse::<IpAddr>().ok()).last())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
}

fn request_client_ip(request: &Request<Body>) -> Option<IpAddr> {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip());
    if peer.is_none_or(|address| address.is_loopback()) {
        if let Some(forwarded) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
        {
            if let Some(parsed) = forwarded
                .split(',')
                .filter_map(|part| part.trim().parse::<IpAddr>().ok())
                .last()
            {
                return Some(parsed);
            }
        }
    }
    peer
}

async fn is_telangana_ip(state: &AppState, ip: IpAddr) -> Result<bool> {
    if ip.is_loopback() {
        return Ok(true);
    }
    let now = Instant::now();
    if let Some(entry) = state.geo_cache.read().await.get(&ip).cloned() {
        if entry.expires_at > now {
            return Ok(entry.allowed);
        }
    }

    // ipapi.co receives the public IP address to estimate its region.
    let url = format!("https://ipapi.co/{ip}/json/");
    let response = state
        .http
        .get(url)
        .send()
        .await
        .context("IP geolocation provider request failed")?;
    if !response.status().is_success() {
        return Err(anyhow!("IP geolocation provider returned HTTP {}", response.status()));
    }
    let geo: IpGeoResponse = response
        .json()
        .await
        .context("could not parse IP geolocation response")?;
    let country = geo.country_code.unwrap_or_default();
    let region = geo.region.unwrap_or_default();
    let region_code = geo.region_code.unwrap_or_default();
    let allowed = country.eq_ignore_ascii_case("IN")
        && (region.trim().eq_ignore_ascii_case("telangana")
            || region_code.trim().eq_ignore_ascii_case("TG")
            || region_code.trim().eq_ignore_ascii_case("TS"));
    state.geo_cache.write().await.insert(
        ip,
        GeoCacheEntry {
            allowed,
            expires_at: now + Duration::from_secs(15 * 60),
        },
    );
    Ok(allowed)
}

async fn region_guard(State(state): State<AppState>, request: Request<Body>, next: Next) -> Response {
    if request.uri().path() == "/api/health" || !state.telangana_only {
        return next.run(request).await;
    }
    let Some(ip) = request_client_ip(&request) else {
        return api_error(StatusCode::FORBIDDEN, "Unable to verify connection region.");
    };
    if ip.is_loopback() {
        return next.run(request).await;
    }
    match is_telangana_ip(&state, ip).await {
        Ok(true) => next.run(request).await,
        Ok(false) => deny_region_and_revoke(&state, request.headers()).await,
        Err(error) => {
            warn!(error = %error, "IP region lookup failed; denying request");
            api_error(
                StatusCode::FORBIDDEN,
                "Could not verify your location. Try again later or use the private recovery connection.",
            )
        }
    }
}

async fn deny_region_and_revoke(state: &AppState, headers: &HeaderMap) -> Response {
    if let Some(token) = cookie_token(headers) {
        let _ = sqlx::query("DELETE FROM sessions WHERE token_hash=?")
            .bind(hash_token(&token))
            .execute(&state.db)
            .await;
    }
    let mut response = api_error(
        StatusCode::FORBIDDEN,
        "NEXUS access is restricted to public IP addresses geolocated in Telangana, India. Your session has been revoked.",
    );
    clear_session_cookie(&mut response, state.cookie_secure);
    response
}

async fn require_auth(State(state): State<AppState>, request: Request<Body>, next: Next) -> Response {
    let session = match load_session(&state, request.headers()).await {
        Ok(Some(session)) => session,
        Ok(None) => return api_error(StatusCode::UNAUTHORIZED, "Please sign in to continue."),
        Err(error) => {
            warn!(error = %error, "session lookup failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify session.");
        }
    };

    let method_name = request.method().as_str();
    let method_is_safe = method_name == "GET" || method_name == "HEAD" || method_name == "OPTIONS";
    if !method_is_safe {
        let provided = request
            .headers()
            .get("x-csrf-token")
            .and_then(|value| value.to_str().ok());
        if provided != Some(session.csrf_token.as_str()) {
            return api_error(StatusCode::FORBIDDEN, "CSRF validation failed.");
        }
    }
    next.run(request).await
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; object-src 'none'; base-uri 'self'; frame-ancestors 'none'; form-action 'self'"),
    );
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    response
}

async fn load_session(state: &AppState, headers: &HeaderMap) -> std::result::Result<Option<SessionData>, sqlx::Error> {
    let Some(token) = cookie_token(headers) else { return Ok(None) };
    let token_hash = hash_token(&token);
    let row = sqlx::query(
        "SELECT u.id, u.email, u.role, s.csrf_token FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=? AND s.expires_at>?",
    )
    .bind(token_hash)
    .bind(unix_time())
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|row| SessionData {
        user: SafeUser { id: row.get("id"), email: row.get("email"), role: row.get("role") },
        csrf_token: row.get("csrf_token"),
    }))
}

async fn create_session(state: &AppState, user_id: i64) -> Result<(String, String)> {
    let token = random_hex(32);
    let csrf = random_hex(32);
    let now = unix_time();
    sqlx::query("INSERT INTO sessions(token_hash, user_id, csrf_token, created_at, expires_at) VALUES(?, ?, ?, ?, ?)")
        .bind(hash_token(&token))
        .bind(user_id)
        .bind(&csrf)
        .bind(now)
        .bind(now + SESSION_TTL_SECONDS)
        .execute(&state.db)
        .await
        .context("could not create session")?;
    Ok((token, csrf))
}

fn add_session_cookie(response: &mut Response, token: &str, secure: bool) {
    let mut cookie = format!("nexus_session={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={SESSION_TTL_SECONDS}");
    if secure { cookie.push_str("; Secure"); }
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
}

fn clear_session_cookie(response: &mut Response, secure: bool) {
    let suffix = if secure { "; Secure" } else { "" };
    let cookie = format!("nexus_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{suffix}");
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
}

async fn check_auth_rate_limit(state: &AppState, headers: &HeaderMap) -> bool {
    let ip = request_ip_from_headers(headers).to_string();
    let key = format!("auth:{ip}");
    let now = Instant::now();
    let cutoff = now - Duration::from_secs(AUTH_RATE_WINDOW_SECONDS);
    let mut map = state.auth_attempts.lock().await;
    if map.len() > 4096 {
        map.retain(|_, entries| entries.back().is_some_and(|last| *last > cutoff));
    }
    let attempts = map.entry(key).or_default();
    while attempts.front().is_some_and(|time| *time <= cutoff) {
        attempts.pop_front();
    }
    if attempts.len() >= AUTH_RATE_MAX_ATTEMPTS {
        return true;
    }
    attempts.push_back(now);
    false
}

async fn index() -> impl IntoResponse { Html(INDEX_HTML) }
async fn stylesheet() -> impl IntoResponse { ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], APP_CSS) }
async fn javascript() -> impl IntoResponse { ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8")], APP_JS) }

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    sqlx::query("SELECT 1").execute(&state.db).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(HealthResponse { status: "ok", database: "ok", version: env!("CARGO_PKG_VERSION") }))
}

async fn auth_status(State(state): State<AppState>) -> Response {
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users").fetch_one(&state.db).await {
        Ok(count) => Json(AuthStatus {
            initialized: count > 0,
            region_restriction: if state.telangana_only { "Telangana, India" } else { "disabled" },
        }).into_response(),
        Err(error) => {
            warn!(error = %error, "could not read authentication status");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not read authentication status.")
        }
    }
}

async fn register(State(state): State<AppState>, headers: HeaderMap, Json(credentials): Json<Credentials>) -> Response {
    if check_auth_rate_limit(&state, &headers).await {
        return api_error(StatusCode::TOO_MANY_REQUESTS, "Too many authentication attempts. Try again in 15 minutes.");
    }
    let Some(email) = normalize_email(&credentials.email) else {
        return api_error(StatusCode::BAD_REQUEST, "Enter a valid email address.");
    };
    if credentials.password.len() < 12 || credentials.password.len() > 128 {
        return api_error(StatusCode::BAD_REQUEST, "Password must be between 12 and 128 bytes.");
    }
    let encoded_password = match hash_password(&credentials.password) {
        Ok(hash) => hash,
        Err(error) => {
            warn!(error = %error, "password hashing failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create account.");
        }
    };

    // BEGIN IMMEDIATE serializes first-account creation so only one concurrent
    // registration can receive the admin role.
    let mut connection = match state.db.acquire().await {
        Ok(connection) => connection,
        Err(error) => {
            warn!(error = %error, "database connection failed during registration");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create account.");
        }
    };
    if let Err(error) = sqlx::query("BEGIN IMMEDIATE").execute(&mut *connection).await {
        warn!(error = %error, "could not begin registration transaction");
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "Database is busy. Try again.");
    }
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users").fetch_one(&mut *connection).await;
    let count = match count {
        Ok(count) => count,
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            warn!(error = %error, "could not count users during registration");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create account.");
        }
    };
    let role = if count == 0 { "admin" } else { "user" };
    let inserted = sqlx::query("INSERT INTO users(email, password_hash, role) VALUES(?, ?, ?)")
        .bind(&email)
        .bind(&encoded_password)
        .bind(role)
        .execute(&mut *connection)
        .await;
    let result = match inserted {
        Ok(result) => result,
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            if error.to_string().contains("UNIQUE constraint failed") {
                return api_error(StatusCode::CONFLICT, "An account with that email already exists.");
            }
            warn!(error = %error, "account insert failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create account.");
        }
    };
    if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
        warn!(error = %error, "registration transaction commit failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not finish account creation.");
    }
    let user = SafeUser { id: result.last_insert_rowid(), email: email.clone(), role: role.to_string() };
    let event_message = if role == "admin" { "Initial administrator account created" } else { "A user account was registered" };
    let _ = sqlx::query("INSERT INTO events(level, message) VALUES('info', ?)").bind(event_message).execute(&state.db).await;
    let (token, csrf_token) = match create_session(&state, user.id).await {
        Ok(pair) => pair,
        Err(error) => {
            warn!(error = %error, "session creation failed after account registration");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Account created, but session setup failed. Sign in.");
        }
    };
    let mut response = (StatusCode::CREATED, Json(AuthPayload { user, csrf_token })).into_response();
    add_session_cookie(&mut response, &token, state.cookie_secure);
    response
}

async fn login(State(state): State<AppState>, headers: HeaderMap, Json(credentials): Json<Credentials>) -> Response {
    if check_auth_rate_limit(&state, &headers).await {
        return api_error(StatusCode::TOO_MANY_REQUESTS, "Too many authentication attempts. Try again in 15 minutes.");
    }
    let Some(email) = normalize_email(&credentials.email) else {
        return api_error(StatusCode::UNAUTHORIZED, "Invalid email or password.");
    };
    let row = match sqlx::query("SELECT id, email, password_hash, role FROM users WHERE email = ? COLLATE NOCASE")
        .bind(&email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return api_error(StatusCode::UNAUTHORIZED, "Invalid email or password."),
        Err(error) => {
            warn!(error = %error, "user lookup failed during login");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not sign in.");
        }
    };
    let password_hash: String = row.get("password_hash");
    if !password_matches(&credentials.password, &password_hash) {
        return api_error(StatusCode::UNAUTHORIZED, "Invalid email or password.");
    }
    let user = SafeUser { id: row.get("id"), email: row.get("email"), role: row.get("role") };
    let (token, csrf_token) = match create_session(&state, user.id).await {
        Ok(pair) => pair,
        Err(error) => {
            warn!(error = %error, "session creation failed during login");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create sign-in session.");
        }
    };
    let mut response = Json(AuthPayload { user, csrf_token }).into_response();
    add_session_cookie(&mut response, &token, state.cookie_secure);
    response
}

async fn auth_me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match load_session(&state, &headers).await {
        Ok(Some(session)) => Json(AuthPayload { user: session.user, csrf_token: session.csrf_token }).into_response(),
        Ok(None) => api_error(StatusCode::UNAUTHORIZED, "Please sign in to continue."),
        Err(error) => {
            warn!(error = %error, "session lookup failed");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify session.")
        }
    }
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match load_session(&state, &headers).await {
        Ok(session) => session,
        Err(error) => {
            warn!(error = %error, "session lookup failed during logout");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not sign out.");
        }
    };
    if let Some(session) = session {
        let provided = headers.get("x-csrf-token").and_then(|value| value.to_str().ok());
        if provided != Some(session.csrf_token.as_str()) {
            return api_error(StatusCode::FORBIDDEN, "CSRF validation failed.");
        }
        if let Some(token) = cookie_token(&headers) {
            if let Err(error) = sqlx::query("DELETE FROM sessions WHERE token_hash=?")
                .bind(hash_token(&token)).execute(&state.db).await
            {
                warn!(error = %error, "session deletion failed during logout");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not sign out.");
            }
        }
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    clear_session_cookie(&mut response, state.cookie_secure);
    response
}

async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    let (ip, port) = bind_config().expect("binding was validated during startup");
    Json(InfoResponse {
        name: "NEXUS Server",
        version: env!("CARGO_PKG_VERSION"),
        mode: "phase-2-authenticated-local-only",
        uptime_seconds: state.started.elapsed().as_secs(),
        bind_address: format!("{ip}:{port}"),
    })
}

async fn metrics() -> Json<MetricsResponse> {
    let mut system = System::new_all();
    system.refresh_all();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_percent = if total_memory == 0 { 0.0 } else { (used_memory as f64 / total_memory as f64 * 100.0).clamp(0.0, 100.0) };
    let disks = Disks::new_with_refreshed_list();
    let root_disk = disks.list().iter().find(|disk| disk.mount_point() == Path::new("/")).or_else(|| disks.list().first());
    let disk_metrics = root_disk.map(|disk| {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        DiskMetrics {
            mount_point: disk.mount_point().to_string_lossy().into_owned(),
            total_bytes: total,
            available_bytes: available,
            used_bytes: used,
            used_percent: if total == 0 { 0.0 } else { (used as f64 / total as f64 * 100.0).clamp(0.0, 100.0) },
        }
    });
    Json(MetricsResponse {
        cpu_percent: system.global_cpu_usage().clamp(0.0, 100.0),
        memory: MemoryMetrics { total_bytes: total_memory, used_bytes: used_memory, used_percent: memory_percent },
        disk: disk_metrics,
        system_uptime_seconds: System::uptime(),
        collected_at_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default(),
    })
}

async fn events(State(state): State<AppState>) -> Result<Json<Vec<Event>>, StatusCode> {
    let rows = sqlx::query("SELECT id, created_at, level, message FROM events ORDER BY id DESC LIMIT 12")
        .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|row| Event {
        id: row.get("id"), created_at: row.get("created_at"), level: row.get("level"), message: row.get("message"),
    }).collect()))
}


// Phase B backend foundation. These APIs persist user-owned bot configuration only.
// Executing user code remains disabled until the separately restricted supervisor is installed.
#[derive(Clone, Serialize)]
struct BotRecord {
    id: i64,
    name: String,
    description: Option<String>,
    runtime: String,
    runtime_version: String,
    entrypoint: String,
    cpu_limit_millicores: i64,
    memory_limit_mb: i64,
    desired_state: String,
    observed_state: String,
    created_at: String,
    updated_at: String,
    last_activity_at: Option<String>,
}

#[derive(Deserialize)]
struct CreateBotRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    runtime: String,
    runtime_version: String,
    entrypoint: String,
    #[serde(default)]
    cpu_limit_millicores: Option<i64>,
    #[serde(default)]
    memory_limit_mb: Option<i64>,
}

#[derive(Deserialize)]
struct UpdateBotRequest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    entrypoint: Option<String>,
    #[serde(default)]
    cpu_limit_millicores: Option<i64>,
    #[serde(default)]
    memory_limit_mb: Option<i64>,
}

#[derive(Serialize)]
struct BotEventRecord {
    id: i64,
    event_type: String,
    message: String,
    created_at: String,
}

#[derive(Serialize)]
struct RuntimeCapabilities {
    metadata_api: bool,
    execution_enabled: bool,
    supervisor_status: &'static str,
    message: &'static str,
}

fn bot_from_row(row: SqliteRow) -> BotRecord {
    BotRecord {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        runtime: row.get("runtime"),
        runtime_version: row.get("runtime_version"),
        entrypoint: row.get("entrypoint"),
        cpu_limit_millicores: row.get("cpu_limit_millicores"),
        memory_limit_mb: row.get("memory_limit_mb"),
        desired_state: row.get("desired_state"),
        observed_state: row.get("observed_state"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_activity_at: row.get("last_activity_at"),
    }
}

async fn current_user_id(state: &AppState, headers: &HeaderMap) -> std::result::Result<i64, Response> {
    match load_session(state, headers).await {
        Ok(Some(session)) => Ok(session.user.id),
        Ok(None) => Err(api_error(StatusCode::UNAUTHORIZED, "Please sign in to continue.")),
        Err(error) => {
            warn!(error = %error, "bot API session lookup failed");
            Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify session."))
        }
    }
}

fn valid_bot_name(value: &str) -> bool {
    let v = value.trim();
    (2..=48).contains(&v.len())
        && v == value
        && v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
        && v.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && !v.ends_with('.')
}

fn valid_entrypoint(runtime: &str, value: &str) -> bool {
    if value.is_empty() || value.len() > 160 || value.starts_with('/') || value.contains('\\') || value.contains(':') || value.contains('\0') {
        return false;
    }
    let parts: Vec<&str> = value.split('/').collect();
    if parts.iter().any(|p| p.is_empty() || *p == "." || *p == ".." || !p.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))) {
        return false;
    }
    let extension_ok = match runtime {
        "python" => value.ends_with(".py"),
        "node" => value.ends_with(".js") || value.ends_with(".mjs") || value.ends_with(".cjs"),
        _ => false,
    };
    extension_ok && !value.starts_with('.')
}

fn validate_runtime(runtime: &str, version: &str) -> bool {
    matches!((runtime, version), ("python", "3.11") | ("python", "3.12") | ("node", "20") | ("node", "22"))
}

fn validate_limits(cpu: i64, memory: i64) -> bool {
    (100..=2000).contains(&cpu) && (128..=2048).contains(&memory)
}

async fn get_owned_bot(state: &AppState, owner_id: i64, bot_id: i64) -> Result<Option<BotRecord>, sqlx::Error> {
    let row = sqlx::query("SELECT id,name,description,runtime,runtime_version,entrypoint,cpu_limit_millicores,memory_limit_mb,desired_state,observed_state,created_at,updated_at,last_activity_at FROM bots WHERE id=? AND owner_user_id=?")
        .bind(bot_id).bind(owner_id).fetch_optional(&state.db).await?;
    Ok(row.map(bot_from_row))
}

async fn write_bot_event(state: &AppState, owner_id: i64, bot_id: i64, event_type: &str, message: &str) {
    if let Err(error) = sqlx::query("INSERT INTO bot_events(bot_id,owner_user_id,event_type,message) VALUES(?,?,?,?)")
        .bind(bot_id).bind(owner_id).bind(event_type).bind(message).execute(&state.db).await {
        warn!(error = %error, bot_id, "could not record bot event");
    } else if let Err(error) = sqlx::query("UPDATE bots SET last_activity_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=? AND owner_user_id=?")
        .bind(bot_id).bind(owner_id).execute(&state.db).await {
        warn!(error = %error, bot_id, "could not update bot activity timestamp");
    }
}

async fn list_bots(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    let rows = match sqlx::query("SELECT id,name,description,runtime,runtime_version,entrypoint,cpu_limit_millicores,memory_limit_mb,desired_state,observed_state,created_at,updated_at,last_activity_at FROM bots WHERE owner_user_id=? ORDER BY id DESC LIMIT 200")
        .bind(owner_id).fetch_all(&state.db).await {
        Ok(rows) => rows,
        Err(error) => { warn!(error = %error, "bot list query failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not load bots."); }
    };
    Json(rows.into_iter().map(bot_from_row).collect::<Vec<_>>()).into_response()
}

async fn create_bot(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<CreateBotRequest>) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    let name = input.name.trim();
    let description = input.description.as_deref().map(str::trim).filter(|v| !v.is_empty());
    let cpu = input.cpu_limit_millicores.unwrap_or(1000);
    let memory = input.memory_limit_mb.unwrap_or(512);
    if !valid_bot_name(name) { return api_error(StatusCode::BAD_REQUEST, "Bot name must be 2–48 characters and use letters, numbers, spaces, dots, underscores or hyphens."); }
    if description.is_some_and(|v| v.len() > 240) { return api_error(StatusCode::BAD_REQUEST, "Description must be 240 characters or fewer."); }
    if !validate_runtime(&input.runtime, &input.runtime_version) { return api_error(StatusCode::BAD_REQUEST, "Unsupported runtime/version combination."); }
    if !valid_entrypoint(&input.runtime, &input.entrypoint) { return api_error(StatusCode::BAD_REQUEST, "Entrypoint must be a safe relative Python or JavaScript file path."); }
    if !validate_limits(cpu, memory) { return api_error(StatusCode::BAD_REQUEST, "Resource limits are outside the permitted range."); }
    let user_count = match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bots WHERE owner_user_id=?").bind(owner_id).fetch_one(&state.db).await {
        Ok(count) => count,
        Err(error) => { warn!(error = %error, "bot quota query failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify bot quota."); }
    };
    if user_count >= MAX_BOTS_PER_USER { return api_error(StatusCode::TOO_MANY_REQUESTS, "Bot quota reached for this account."); }
    let global_count = match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bots").fetch_one(&state.db).await {
        Ok(count) => count,
        Err(error) => { warn!(error = %error, "global bot quota query failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify hosting capacity."); }
    };
    if global_count >= MAX_BOTS_GLOBAL { return api_error(StatusCode::TOO_MANY_REQUESTS, "NEXUS bot capacity is currently full."); }
    let result = sqlx::query("INSERT INTO bots(owner_user_id,name,description,runtime,runtime_version,entrypoint,cpu_limit_millicores,memory_limit_mb,desired_state,observed_state) VALUES(?,?,?,?,?,?,?,?, 'stopped','stopped')")
        .bind(owner_id).bind(name).bind(description).bind(&input.runtime).bind(&input.runtime_version).bind(&input.entrypoint).bind(cpu).bind(memory).execute(&state.db).await;
    let bot_id = match result { Ok(result) => result.last_insert_rowid(), Err(error) => { warn!(error = %error, "bot creation failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create bot configuration."); } };
    write_bot_event(&state, owner_id, bot_id, "created", "Bot configuration created; runtime not provisioned.").await;
    match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(Some(bot)) => (StatusCode::CREATED, Json(bot)).into_response(),
        _ => api_error(StatusCode::INTERNAL_SERVER_ERROR, "Bot was created but could not be loaded."),
    }
}

async fn get_bot(State(state): State<AppState>, headers: HeaderMap, AxumPath(bot_id): AxumPath<i64>) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(Some(bot)) => Json(bot).into_response(),
        Ok(None) => api_error(StatusCode::NOT_FOUND, "Bot not found."),
        Err(error) => { warn!(error = %error, "bot detail query failed"); api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not load bot.") }
    }
}

async fn update_bot(State(state): State<AppState>, headers: HeaderMap, AxumPath(bot_id): AxumPath<i64>, Json(input): Json<UpdateBotRequest>) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    let mut bot = match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(Some(bot)) => bot,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, "Bot not found."),
        Err(error) => { warn!(error = %error, "bot update lookup failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not load bot."); }
    };
    if let Some(name) = input.name { if !valid_bot_name(name.trim()) { return api_error(StatusCode::BAD_REQUEST, "Invalid bot name."); } bot.name = name.trim().to_string(); }
    if let Some(description) = input.description { let desc = description.trim(); if desc.len() > 240 { return api_error(StatusCode::BAD_REQUEST, "Description must be 240 characters or fewer."); } bot.description = if desc.is_empty() { None } else { Some(desc.to_string()) }; }
    if let Some(entrypoint) = input.entrypoint { if !valid_entrypoint(&bot.runtime, &entrypoint) { return api_error(StatusCode::BAD_REQUEST, "Invalid entrypoint path for this runtime."); } bot.entrypoint = entrypoint; }
    if let Some(cpu) = input.cpu_limit_millicores { bot.cpu_limit_millicores = cpu; }
    if let Some(memory) = input.memory_limit_mb { bot.memory_limit_mb = memory; }
    if !validate_limits(bot.cpu_limit_millicores, bot.memory_limit_mb) { return api_error(StatusCode::BAD_REQUEST, "Resource limits are outside the permitted range."); }
    if bot.observed_state != "stopped" { return api_error(StatusCode::CONFLICT, "Configuration can only be edited while the bot is stopped."); }
    if let Err(error) = sqlx::query("UPDATE bots SET name=?,description=?,entrypoint=?,cpu_limit_millicores=?,memory_limit_mb=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=? AND owner_user_id=?")
        .bind(&bot.name).bind(&bot.description).bind(&bot.entrypoint).bind(bot.cpu_limit_millicores).bind(bot.memory_limit_mb).bind(bot_id).bind(owner_id).execute(&state.db).await {
        warn!(error = %error, "bot update failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not update bot.");
    }
    write_bot_event(&state, owner_id, bot_id, "configuration_updated", "Bot configuration updated.").await;
    match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(Some(updated)) => Json(updated).into_response(),
        _ => api_error(StatusCode::INTERNAL_SERVER_ERROR, "Bot updated but could not be reloaded."),
    }
}

async fn delete_bot(State(state): State<AppState>, headers: HeaderMap, AxumPath(bot_id): AxumPath<i64>) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(None) => return api_error(StatusCode::NOT_FOUND, "Bot not found."),
        Err(error) => { warn!(error = %error, "bot deletion lookup failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify bot."); }
        Ok(Some(bot)) if bot.observed_state != "stopped" && bot.observed_state != "failed" => return api_error(StatusCode::CONFLICT, "Only stopped or failed bots can be deleted."),
        Ok(Some(_)) => {},
    }
    // Do not delete records yet: a runtime-aware cleanup supervisor is required to avoid orphaned resources.
    api_error(StatusCode::SERVICE_UNAVAILABLE, "Deletion is disabled until the isolated runtime supervisor can confirm resource cleanup.")
}

async fn bot_activity(State(state): State<AppState>, headers: HeaderMap, AxumPath(bot_id): AxumPath<i64>) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(Some(_)) => {},
        Ok(None) => return api_error(StatusCode::NOT_FOUND, "Bot not found."),
        Err(error) => { warn!(error = %error, "bot activity ownership check failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not load bot activity."); }
    }
    let rows = match sqlx::query("SELECT id,event_type,message,created_at FROM bot_events WHERE bot_id=? AND owner_user_id=? ORDER BY id DESC LIMIT 100")
        .bind(bot_id).bind(owner_id).fetch_all(&state.db).await {
        Ok(rows) => rows,
        Err(error) => { warn!(error = %error, "bot activity query failed"); return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not load bot activity."); }
    };
    let events = rows.into_iter().map(|row| BotEventRecord { id: row.get("id"), event_type: row.get("event_type"), message: row.get("message"), created_at: row.get("created_at") }).collect::<Vec<_>>();
    Json(events).into_response()
}

async fn runtime_capabilities() -> Json<RuntimeCapabilities> {
    Json(RuntimeCapabilities { metadata_api: true, execution_enabled: false, supervisor_status: "not_installed", message: "Bot configuration APIs are available; execution, files, secrets, dependencies and lifecycle actions remain disabled until the isolated supervisor is installed and verified." })
}

async fn unavailable_runtime_action(State(state): State<AppState>, headers: HeaderMap, AxumPath(bot_id): AxumPath<i64>) -> Response {
    let owner_id = match current_user_id(&state, &headers).await { Ok(id) => id, Err(response) => return response };
    match get_owned_bot(&state, owner_id, bot_id).await {
        Ok(Some(_)) => api_error(StatusCode::SERVICE_UNAVAILABLE, "Bot runtime is not configured. No code was executed and no lifecycle change was made."),
        Ok(None) => api_error(StatusCode::NOT_FOUND, "Bot not found."),
        Err(error) => { warn!(error = %error, "runtime action ownership check failed"); api_error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify bot ownership.") }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler"); };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install terminate signal handler").recv().await;
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
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .compact()
        .init();

    let (bind_ip, port) = bind_config()?;
    let directory = data_dir()?;
    std::fs::create_dir_all(&directory).with_context(|| format!("could not create data directory {}", directory.display()))?;
    let canonical_dir = std::fs::canonicalize(&directory).with_context(|| format!("could not resolve data directory {}", directory.display()))?;
    let database_path = canonical_dir.join("nexus.sqlite3");
    let db = initialize_database(&database_path).await?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .user_agent(concat!("NEXUS-Server/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("could not initialize HTTP client for IP region checks")?;
    let state = AppState {
        db,
        started: Instant::now(),
        http,
        geo_cache: Arc::new(RwLock::new(HashMap::new())),
        auth_attempts: Arc::new(Mutex::new(HashMap::new())),
        telangana_only: env_bool("NEXUS_TELANGANA_ONLY", true)?,
        cookie_secure: env_bool("NEXUS_COOKIE_SECURE", false)?,
    };

    let protected_api = Router::new()
        .route("/api/info", get(info))
        .route("/api/metrics", get(metrics))
        .route("/api/events", get(events))
        .route("/api/bots/runtime-capabilities", get(runtime_capabilities))
        .route("/api/bots", get(list_bots).post(create_bot))
        .route("/api/bots/{id}", get(get_bot).patch(update_bot).delete(delete_bot))
        .route("/api/bots/{id}/activity", get(bot_activity))
        .route("/api/bots/{id}/start", post(unavailable_runtime_action))
        .route("/api/bots/{id}/stop", post(unavailable_runtime_action))
        .route("/api/bots/{id}/restart", post(unavailable_runtime_action))
        .route("/api/bots/{id}/dependencies/install", post(unavailable_runtime_action))
        .route("/api/bots/{id}/logs", get(unavailable_runtime_action))
        .route("/api/bots/{id}/files", get(unavailable_runtime_action))
        .route("/api/bots/{id}/environment", get(unavailable_runtime_action))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let app = Router::new()
        .route("/", get(index))
        .route("/assets/app.css", get(stylesheet))
        .route("/assets/app.js", get(javascript))
        .route("/api/health", get(health))
        .route("/api/auth/status", get(auth_status))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(auth_me))
        .route("/api/auth/logout", post(logout))
        .merge(protected_api)
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), region_guard))
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http());

    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address).await.with_context(|| format!("could not bind to {address}"))?;
    info!(bind = %address, data_dir = %canonical_dir.display(), telangana_only = state.telangana_only, "NEXUS Phase 2 starting");
    warn!("create the first admin account over a private connection before enabling any public tunnel");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("NEXUS HTTP server failed")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{hash_password, normalize_email, password_matches, valid_bot_name, valid_entrypoint, validate_limits, validate_runtime};

    #[test]
    fn password_hash_verifies_without_storing_plaintext() {
        let password = "a-long-test-password-123";
        let hash = hash_password(password).expect("password hashing should work");
        assert_ne!(hash, password);
        assert!(password_matches(password, &hash));
        assert!(!password_matches("incorrect", &hash));
    }

    #[test]
    fn email_normalization_is_case_insensitive() {
        assert_eq!(normalize_email("  Admin@Example.com "), Some("admin@example.com".to_string()));
        assert_eq!(normalize_email("not-an-email"), None);
    }

    #[test]
    fn bot_names_and_entrypoints_reject_unsafe_values() {
        assert!(valid_bot_name("megabot_01"));
        assert!(!valid_bot_name(" ../bad"));
        assert!(!valid_bot_name("name\nother"));
        assert!(valid_entrypoint("python", "main.py"));
        assert!(valid_entrypoint("node", "src/index.js"));
        assert!(!valid_entrypoint("python", "../main.py"));
        assert!(!valid_entrypoint("python", "/etc/passwd.py"));
        assert!(!valid_entrypoint("node", "src/../index.js"));
        assert!(!valid_entrypoint("node", "index.sh"));
    }

    #[test]
    fn runtime_and_resource_allowlists_are_enforced() {
        assert!(validate_runtime("python", "3.12"));
        assert!(validate_runtime("node", "22"));
        assert!(!validate_runtime("python", "latest"));
        assert!(!validate_runtime("node", "20.999"));
        assert!(validate_limits(1000, 512));
        assert!(!validate_limits(5000, 512));
        assert!(!validate_limits(1000, 8192));
    }
}
