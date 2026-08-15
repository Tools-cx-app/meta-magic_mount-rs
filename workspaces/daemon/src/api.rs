// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{path::PathBuf, sync::Arc};

use api::{
    ApiConfig, ApiError, DeviceInfo, ErrorDetail, ErrorResponse, OpenLinkRequest, Status,
    SystemInfo,
};
use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::{FromRequest, Request, State, rejection::JsonRejection},
    http::{
        HeaderValue, Method, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tokio::process::Command;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::{
    auth::is_authorized,
    config::{ConfigError, Store},
    defs, scanner,
};

#[derive(Clone)]
pub struct AppState {
    store: Store,
    modules_path: PathBuf,
    token: Arc<str>,
    actions: Arc<dyn SystemActions>,
    snapshot: Arc<RwLock<Option<Snapshot>>>,
}

#[derive(Clone)]
struct Snapshot {
    config: ApiConfig,
    modules: Vec<api::Module>,
}

impl AppState {
    pub fn production(token: impl Into<Arc<str>>) -> Self {
        Self::new(
            Store::new(defs::CONFIG_FILE, defs::CUSTOM_LIST_PATH),
            defs::MODULE_PATH,
            token,
            Arc::new(AndroidSystemActions),
        )
    }

    pub fn new(
        store: Store,
        modules_path: impl Into<PathBuf>,
        token: impl Into<Arc<str>>,
        actions: Arc<dyn SystemActions>,
    ) -> Self {
        Self {
            store,
            modules_path: modules_path.into(),
            token: token.into(),
            actions,
            snapshot: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn initialize(self) -> anyhow::Result<Self> {
        let snapshot = load_snapshot(&self.store, &self.modules_path).await?;
        *self.snapshot.write().await = Some(snapshot);
        Ok(self)
    }
}

async fn load_snapshot(store: &Store, modules_path: &PathBuf) -> anyhow::Result<Snapshot> {
    let config = store.load().await?;
    let modules = scanner::list_modules(modules_path, &config.partitions).await;
    Ok(Snapshot { config, modules })
}

#[async_trait]
pub trait SystemActions: Send + Sync {
    async fn status(&self) -> Status;
    async fn open_link(&self, url: &str) -> anyhow::Result<()>;
    async fn reboot(&self) -> anyhow::Result<()>;
}

pub struct AndroidSystemActions;

async fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().await.ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|value| !value.is_empty())
}

async fn successful_command(command: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(command).args(args).status().await?;
    anyhow::ensure!(status.success(), "{command} exited with {status}");
    Ok(())
}

#[async_trait]
impl SystemActions for AndroidSystemActions {
    async fn status(&self) -> Status {
        let (model, kernel, selinux) = tokio::join!(
            command_output("getprop", &["ro.product.model"]),
            command_output("uname", &["-r"]),
            command_output("getenforce", &[]),
        );
        Status {
            version: env!("CARGO_PKG_VERSION").into(),
            device: DeviceInfo { model },
            system: SystemInfo { kernel, selinux },
        }
    }

    async fn open_link(&self, url: &str) -> anyhow::Result<()> {
        successful_command(
            "am",
            &["start", "-a", "android.intent.action.VIEW", "-d", url],
        )
        .await
    }

    async fn reboot(&self) -> anyhow::Result<()> {
        if let Err(svc_error) = successful_command("svc", &["power", "reboot"]).await {
            successful_command("reboot", &[])
                .await
                .map_err(|fallback_error| {
                    anyhow::anyhow!(
                        "svc reboot failed: {svc_error}; fallback reboot failed: {fallback_error}"
                    )
                })?;
        }
        Ok(())
    }
}

struct ApiFailure(StatusCode, ApiError, &'static str);

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: self.1,
                    message: self.2.into(),
                },
            }),
        )
            .into_response()
    }
}

type ApiResult<T> = Result<T, ApiFailure>;

fn internal(error: impl std::fmt::Display) -> ApiFailure {
    log::error!("API operation failed: {error}");
    ApiFailure(
        StatusCode::INTERNAL_SERVER_ERROR,
        ApiError::Internal,
        "Internal server error",
    )
}

async fn authorize(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    if is_authorized(&state.token, header) {
        next.run(request).await
    } else {
        ApiFailure(
            StatusCode::UNAUTHORIZED,
            ApiError::Unauthorized,
            "Unauthorized",
        )
        .into_response()
    }
}

async fn get_config(State(state): State<AppState>) -> ApiResult<Json<ApiConfig>> {
    state
        .snapshot
        .read()
        .await
        .as_ref()
        .map(|snapshot| Json(snapshot.config.clone()))
        .ok_or_else(|| internal("daemon snapshot is not initialized"))
}

async fn reload(State(state): State<AppState>, request: Request) -> ApiResult<StatusCode> {
    let Json(config) = Json::<ApiConfig>::from_request(request, &state)
        .await
        .map_err(invalid_json)?;
    state
        .store
        .save(config)
        .await
        .map_err(|error| match error {
            ConfigError::Invalid(_) => ApiFailure(
                StatusCode::BAD_REQUEST,
                ApiError::InvalidConfig,
                "Invalid config",
            ),
            ConfigError::Other(error) => internal(error),
        })?;
    let snapshot = load_snapshot(&state.store, &state.modules_path)
        .await
        .map_err(internal)?;
    *state.snapshot.write().await = Some(snapshot);
    Ok(StatusCode::NO_CONTENT)
}

fn invalid_json(error: JsonRejection) -> ApiFailure {
    log::debug!("invalid JSON request: {error}");
    ApiFailure(
        StatusCode::BAD_REQUEST,
        ApiError::InvalidRequest,
        "Invalid request",
    )
}

async fn get_modules(State(state): State<AppState>) -> ApiResult<Json<Vec<api::Module>>> {
    state
        .snapshot
        .read()
        .await
        .as_ref()
        .map(|snapshot| Json(snapshot.modules.clone()))
        .ok_or_else(|| internal("daemon snapshot is not initialized"))
}

async fn get_status(State(state): State<AppState>) -> Json<Status> {
    Json(state.actions.status().await)
}

async fn open_link(State(state): State<AppState>, request: Request) -> ApiResult<StatusCode> {
    let Json(input) = Json::<OpenLinkRequest>::from_request(request, &state)
        .await
        .map_err(invalid_json)?;
    let valid = url::Url::parse(&input.url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some());
    if valid.is_none() {
        return Err(ApiFailure(
            StatusCode::BAD_REQUEST,
            ApiError::InvalidRequest,
            "Invalid URL",
        ));
    }
    state.actions.open_link(&input.url).await.map_err(|error| {
        log::error!("failed to open link: {error}");
        ApiFailure(
            StatusCode::SERVICE_UNAVAILABLE,
            ApiError::Unavailable,
            "System action unavailable",
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reboot(State(state): State<AppState>) -> ApiResult<StatusCode> {
    state.actions.reboot().await.map_err(|error| {
        log::error!("failed to reboot: {error}");
        ApiFailure(
            StatusCode::SERVICE_UNAVAILABLE,
            ApiError::Unavailable,
            "System action unavailable",
        )
    })?;
    Ok(StatusCode::ACCEPTED)
}

async fn not_found() -> ApiFailure {
    ApiFailure(StatusCode::NOT_FOUND, ApiError::NotFound, "Not found")
}

async fn method_not_allowed() -> ApiFailure {
    ApiFailure(
        StatusCode::METHOD_NOT_ALLOWED,
        ApiError::InvalidRequest,
        "Method not allowed",
    )
}

pub fn router(state: AppState) -> Router {
    let v1 = Router::new()
        .route("/config", get(get_config))
        .route("/modules", get(get_modules))
        .route("/status", get(get_status))
        .route("/actions/open-link", post(open_link))
        .route("/actions/reboot", post(reboot))
        .route("/actions/reload", post(reload))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(from_fn_with_state(state.clone(), authorize))
        .with_state(state);
    Router::new().nest("/api/v1", v1).layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
                origin == "https://mui.kernelsu.org"
            }))
            .allow_private_network(true)
            .allow_methods([Method::GET, Method::PUT, Method::POST])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE]),
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/daemon/api.rs"]
mod tests;
