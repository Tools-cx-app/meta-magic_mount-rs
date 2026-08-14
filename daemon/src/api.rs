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
mod tests {
    use std::{path::Path, sync::Arc};

    use api::{ApiConfig, ApiError, DeviceInfo, ErrorResponse, Status, SystemInfo};
    use async_trait::async_trait;
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{
            Method, Request, StatusCode,
            header::{ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
        },
    };
    use tower::ServiceExt;

    use super::{AppState, SystemActions, router};
    use crate::config::Store;

    #[derive(Default)]
    struct FakeActions {
        fail: bool,
    }

    #[async_trait]
    impl SystemActions for FakeActions {
        async fn status(&self) -> Status {
            Status {
                version: env!("CARGO_PKG_VERSION").into(),
                device: DeviceInfo {
                    model: Some("Test Device".into()),
                },
                system: SystemInfo {
                    kernel: Some("test-kernel".into()),
                    selinux: Some("Enforcing".into()),
                },
            }
        }

        async fn open_link(&self, _url: &str) -> anyhow::Result<()> {
            anyhow::ensure!(!self.fail, "open failed");
            Ok(())
        }

        async fn reboot(&self) -> anyhow::Result<()> {
            anyhow::ensure!(!self.fail, "reboot failed");
            Ok(())
        }
    }

    async fn test_router(fail_actions: bool) -> (Router, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("config.toml"), dir.path().join("custom"));
        store.save(test_config()).await.unwrap();
        let modules = dir.path().join("modules");
        tokio::fs::create_dir(&modules).await.unwrap();
        (
            router(
                AppState::new(
                    store,
                    modules,
                    "test-token",
                    Arc::new(FakeActions { fail: fail_actions }),
                )
                .initialize()
                .await
                .unwrap(),
            ),
            dir,
        )
    }

    fn test_config() -> ApiConfig {
        ApiConfig {
            mountsource: "KSU".into(),
            umount: false,
            partitions: vec![],
            ignore_list: vec![],
            custom_mounts: vec![],
        }
    }

    fn authorized(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(path)
            .header(AUTHORIZATION, "Bearer test-token")
            .body(Body::empty())
            .unwrap()
    }

    async fn error(response: axum::response::Response) -> ApiError {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice::<ErrorResponse>(&body)
            .unwrap()
            .error
            .code
    }

    async fn write_module(root: &Path) {
        let module = root.join("example");
        tokio::fs::create_dir_all(module.join("vendor"))
            .await
            .unwrap();
        tokio::fs::write(
            module.join("module.prop"),
            "id=example\nname=Example\nversion=1\nauthor=Tester\ndescription=Test\n",
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn rejects_missing_or_inexact_auth_with_common_error() {
        let (app, _dir) = test_router(false).await;
        for header in [None, Some("Bearer Test-token"), Some("Basic test-token")] {
            let mut request = Request::get("/api/v1/config").body(Body::empty()).unwrap();
            if let Some(header) = header {
                request
                    .headers_mut()
                    .insert(AUTHORIZATION, header.parse().unwrap());
            }
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(error(response).await, ApiError::Unauthorized);
        }
    }

    #[tokio::test]
    async fn config_round_trip_uses_full_json() {
        let (app, _dir) = test_router(false).await;
        let request = Request::post("/api/v1/actions/reload")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"mountsource":"KSU","umount":true,"partitions":["vendor"],"ignoreList":["/ignored"],"customMounts":[{"source":"/a","target":"/b"}]}"#))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
        let response = app
            .oneshot(authorized(Method::GET, "/api/v1/config"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let config: ApiConfig = serde_json::from_slice(&body).unwrap();
        assert!(config.umount);
        assert_eq!(config.partitions, ["vendor"]);
        assert_eq!(config.ignore_list, ["/ignored"]);
        assert_eq!(config.custom_mounts[0].target, "/b");
    }

    #[tokio::test]
    async fn config_reads_cached_snapshot() {
        let (app, dir) = test_router(false).await;
        tokio::fs::write(
            dir.path().join("config.toml"),
            "mountsource = \"APatch\"\npartitions = []\numount = false\n",
        )
        .await
        .unwrap();
        let response = app
            .oneshot(authorized(Method::GET, "/api/v1/config"))
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let config: ApiConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(config.mountsource, "KSU");
    }

    #[tokio::test]
    async fn reload_updates_cached_snapshot() {
        let (app, _dir) = test_router(false).await;
        let request = Request::post("/api/v1/actions/reload")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"mountsource":"APatch","umount":false,"partitions":[],"ignoreList":[],"customMounts":[]}"#))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
        let response = app
            .oneshot(authorized(Method::GET, "/api/v1/config"))
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let config: ApiConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(config.mountsource, "APatch");
    }

    #[tokio::test]
    async fn failed_reload_keeps_cached_snapshot() {
        let (app, _dir) = test_router(false).await;
        let request = Request::post("/api/v1/actions/reload")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"mountsource":"","umount":false,"partitions":[],"ignoreList":[],"customMounts":[]}"#))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::BAD_REQUEST
        );
        let response = app
            .oneshot(authorized(Method::GET, "/api/v1/config"))
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let config: ApiConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(config.mountsource, "KSU");
    }

    #[tokio::test]
    async fn invalid_config_and_json_use_common_errors() {
        let (app, _dir) = test_router(false).await;
        let invalid = Request::post("/api/v1/actions/reload")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"mountsource":"","umount":false,"partitions":[],"ignoreList":[],"customMounts":[]}"#))
            .unwrap();
        let response = app.clone().oneshot(invalid).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error(response).await, ApiError::InvalidConfig);

        let malformed = Request::post("/api/v1/actions/reload")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let response = app.oneshot(malformed).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error(response).await, ApiError::InvalidRequest);
    }

    #[tokio::test]
    async fn modules_use_configured_partitions() {
        let (app, dir) = test_router(false).await;
        write_module(&dir.path().join("modules")).await;
        let config = Request::post("/api/v1/actions/reload")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"mountsource":"KSU","umount":false,"partitions":["vendor"],"ignoreList":[],"customMounts":[]}"#))
            .unwrap();
        app.clone().oneshot(config).await.unwrap();

        let response = app
            .oneshot(authorized(Method::GET, "/api/v1/modules"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let modules: Vec<api::Module> = serde_json::from_slice(&body).unwrap();
        assert_eq!(modules.len(), 1);
        assert!(modules[0].is_mounted);
    }

    #[tokio::test]
    async fn open_link_accepts_only_hosted_http_urls() {
        let (app, _dir) = test_router(false).await;
        for url in [
            "file:///data/local/tmp/x",
            "https://",
            "javascript:alert(1)",
        ] {
            let request = Request::post("/api/v1/actions/open-link")
                .header(AUTHORIZATION, "Bearer test-token")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(format!(r#"{{"url":"{url}"}}"#)))
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            assert_eq!(error(response).await, ApiError::InvalidRequest);
        }
        let request = Request::post("/api/v1/actions/open-link")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"url":"https://example.com/path"}"#))
            .unwrap();
        assert_eq!(
            app.oneshot(request).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
    }

    #[tokio::test]
    async fn reboot_is_accepted_and_action_failures_are_common_errors() {
        let (app, _dir) = test_router(false).await;
        assert_eq!(
            app.oneshot(authorized(Method::POST, "/api/v1/actions/reboot"))
                .await
                .unwrap()
                .status(),
            StatusCode::ACCEPTED
        );

        let (failing, _dir) = test_router(true).await;
        let response = failing
            .oneshot(authorized(Method::POST, "/api/v1/actions/reboot"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error(response).await, ApiError::Unavailable);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn system_commands_require_successful_exit_status() {
        assert!(super::successful_command("true", &[]).await.is_ok());
        assert!(super::successful_command("false", &[]).await.is_err());
    }

    #[tokio::test]
    async fn unknown_v1_route_returns_common_not_found() {
        let (app, _dir) = test_router(false).await;
        let response = app
            .oneshot(authorized(Method::GET, "/api/v1/missing"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(error(response).await, ApiError::NotFound);
    }

    #[tokio::test]
    async fn unsupported_method_returns_common_error() {
        let (app, _dir) = test_router(false).await;
        let response = app
            .oneshot(authorized(Method::DELETE, "/api/v1/config"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(error(response).await, ApiError::InvalidRequest);
    }

    #[tokio::test]
    async fn put_config_is_removed() {
        let (app, _dir) = test_router(false).await;
        let response = app
            .oneshot(authorized(Method::PUT, "/api/v1/config"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn cors_rejects_other_origins_and_allows_exact_preflight() {
        let (app, _dir) = test_router(false).await;
        let evil = Request::get("/api/v1/config")
            .header(AUTHORIZATION, "Bearer test-token")
            .header(ORIGIN, "https://evil.example")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(evil).await.unwrap();
        assert!(
            response
                .headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );

        let preflight = Request::builder()
            .method(Method::OPTIONS)
            .uri("/api/v1/config")
            .header(ORIGIN, "https://mui.kernelsu.org")
            .header("access-control-request-method", "PUT")
            .header(
                "access-control-request-headers",
                "authorization,content-type",
            )
            .header("access-control-request-private-network", "true")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(preflight).await.unwrap();
        assert!(response.status().is_success());
        assert_eq!(
            response.headers()[ACCESS_CONTROL_ALLOW_ORIGIN],
            "https://mui.kernelsu.org"
        );
        assert_eq!(
            response.headers()["access-control-allow-private-network"],
            "true"
        );
    }
}
