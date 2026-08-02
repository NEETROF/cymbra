// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! SoundFont delivery + admin upload (changes: add-soundfont-delivery,
//! add-soundfont-catalog-db, add-soundfont-back-office-management).
//!
//! `GET /soundfonts/{id}` streams a SoundFont's bytes from the **private** SoundFont
//! object store to any authenticated caller (access tiers were removed; a later change
//! may reintroduce paid gating). `POST /soundfonts/{id}` is the admin upload counterpart
//! (music-scope moderator/admin), storing the bytes + recording the catalog row.
//!
//! The **decision** logic (auth present? font known? admin? valid body?) and range
//! parsing are pure and host-tested; auth (JWT), the catalog repo, and the object-store
//! IO sit behind injectable seams ([`SoundfontAuth`], [`SoundFontRepo`],
//! [`ObjectStorage`]) so the handlers are testable with fakes — no JWT minting, no network.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::http::{HeaderValue, Method};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use cymbra_music::{FontEntry, SoundFontRepo};
use cymbra_platform::{AuthIdentity, guard, token};
use cymbra_storage::{ObjectStorage, StorageError};
use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Largest `.sf2` upload accepted (change: add-soundfont-back-office-management).
/// The Salamander grand (~296 MiB) is the biggest realistic font; 400 MiB gives
/// headroom. The body is read into memory (the object store's `put` is buffer-based),
/// so this cap also bounds per-upload memory.
const MAX_UPLOAD_BYTES: usize = 400 * 1024 * 1024;

/// Whether `bytes` begin with a SoundFont preamble (`RIFF` … `sfbk`). A cheap guard
/// so a non-SoundFont upload is rejected before anything is stored.
fn is_valid_soundfont(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"sfbk"
}

// The SoundFont catalog is now the persisted `music.soundfonts` table (change:
// add-soundfont-catalog-db), resolved through a [`SoundFontRepo`]; the font type
// (`FontEntry`) lives in `cymbra-music` so the delivery route and the ListSoundFonts
// RPC share one definition. Every catalog font is served to any signed-in caller
// (access tiers were removed — a later change may reintroduce paid gating here).

/// The outcome of the pure request decision, before any bytes are read.
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Unauthenticated,
    NotFound,
    Serve(String), // the object key to stream
}

/// Decide a delivery request purely from the already-resolved inputs: authenticate,
/// then require the font to exist. Kept pure (the id → `font` resolution is an async
/// repo read done by the caller) so a refusal never depends on whether the object
/// exists.
pub fn decide(user_id: Option<&str>, font: Option<&FontEntry>) -> Decision {
    if user_id.is_none() {
        return Decision::Unauthenticated;
    }
    match font {
        Some(font) => Decision::Serve(font.object_key.clone()),
        None => Decision::NotFound,
    }
}

/// A parsed single HTTP byte range (`start`, inclusive `end`). Only the common
/// `bytes=start-end` / `bytes=start-` forms are supported; anything else is ignored
/// (served as a full response).
#[derive(Debug, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: Option<u64>,
}

/// Parse a `Range: bytes=…` header into a single [`ByteRange`], or `None` when absent
/// or not a supported single range.
pub fn parse_range(headers: &HeaderMap) -> Option<ByteRange> {
    let raw = headers.get(header::RANGE)?.to_str().ok()?;
    let spec = raw.trim().strip_prefix("bytes=")?;
    // Reject multi-range (comma) and suffix ranges (`-N`) for v1 simplicity.
    if spec.contains(',') {
        return None;
    }
    let (s, e) = spec.split_once('-')?;
    let start: u64 = s.trim().parse().ok()?;
    let end = match e.trim() {
        "" => None,
        v => Some(v.parse().ok()?),
    };
    if let Some(end) = end
        && end < start
    {
        return None;
    }
    Some(ByteRange { start, end })
}

/// Resolve the caller from request headers. Injectable so the handlers are testable
/// without minting a JWT.
pub trait SoundfontAuth: Send + Sync {
    /// The caller's user id (delivery only needs identity), or `None` if
    /// unauthenticated.
    fn identify(&self, headers: &HeaderMap) -> Option<String>;

    /// The caller's full identity incl. roles (the admin upload route needs roles),
    /// or `None` if unauthenticated. Default: no identity (delivery-only doubles).
    fn identify_admin(&self, _headers: &HeaderMap) -> Option<AuthIdentity> {
        None
    }
}

/// Production auth: verify the `Authorization: Bearer` access token against the
/// published signing keys + audience allow-list (the same tokens the gRPC interceptor
/// checks) and return its subject.
pub struct JwtAuth {
    keys: Arc<HashMap<String, DecodingKey>>,
    audiences: Arc<Vec<String>>,
}

impl JwtAuth {
    pub fn new(keys: HashMap<String, DecodingKey>, audiences: Vec<String>) -> Self {
        Self {
            keys: Arc::new(keys),
            audiences: Arc::new(audiences),
        }
    }
}

impl JwtAuth {
    fn verify(&self, headers: &HeaderMap) -> Option<token::Claims> {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))?;
        let auds: Vec<&str> = self.audiences.iter().map(String::as_str).collect();
        token::verify(token, &self.keys, &auds).ok()
    }
}

impl SoundfontAuth for JwtAuth {
    fn identify(&self, headers: &HeaderMap) -> Option<String> {
        self.verify(headers).map(|c| c.sub)
    }

    fn identify_admin(&self, headers: &HeaderMap) -> Option<AuthIdentity> {
        let c = self.verify(headers)?;
        Some(AuthIdentity {
            user_id: c.sub,
            audience: c.aud,
            roles: c.roles,
            roles_by_scope: c.roles_by_scope,
        })
    }
}

/// Router state: the (optional) SoundFont store, the (optional) persisted catalog
/// repo, and the auth seam. A missing store **or** repo means the feature is
/// unconfigured (the route reports 503).
#[derive(Clone)]
pub struct SoundfontState {
    pub store: Option<Arc<dyn ObjectStorage>>,
    pub repo: Option<Arc<dyn SoundFontRepo>>,
    pub auth: Arc<dyn SoundfontAuth>,
}

/// The SoundFont delivery router (`GET /soundfonts/:id`), ready to `.merge()` into the
/// HTTP server. Always mounted; when unconfigured it responds 503.
///
/// `allowed_origins` are the back-office browser origins permitted to fetch fonts
/// cross-origin (the same allow-list as gRPC-web/web-auth). The CORS layer permits the
/// `Authorization` (bearer) and `Range` request headers and exposes the range/length
/// response headers so the browser can range-fetch and cache.
pub fn soundfont_router(state: SoundfontState, allowed_origins: Vec<String>) -> Router {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::RANGE, header::CONTENT_TYPE])
        .expose_headers([
            header::CONTENT_RANGE,
            header::ACCEPT_RANGES,
            header::CONTENT_LENGTH,
        ]);
    Router::new()
        // `GET` streams a font (delivery); `POST` uploads one (admin, change:
        // add-soundfont-back-office-management). The body limit is raised only for the
        // upload; metadata rides in the query so no multipart is needed.
        .route("/soundfonts/:id", get(serve).post(upload))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .layer(cors)
        .with_state(state)
}

/// Metadata for an upload, carried in the query string (URL-encoded, so unicode
/// labels are safe) alongside the raw `.sf2` body.
#[derive(Debug, Deserialize)]
pub struct UploadMeta {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub attribution: String,
    /// Instrument family (e.g. "piano"); defaults to piano when absent.
    #[serde(default)]
    pub instrument: String,
}

/// The pure upload decision, before any store/DB access: authenticate, require a
/// music-scope moderator/admin, then require a valid SoundFont body.
#[derive(Debug, PartialEq, Eq)]
pub enum UploadDecision {
    Unauthenticated,
    Forbidden,
    InvalidBody,
    Accept,
}

/// Decide an upload purely from the resolved identity and whether the body is a
/// SoundFont — kept host-testable and independent of IO.
pub fn decide_upload(identity: Option<&AuthIdentity>, is_valid_sf2: bool) -> UploadDecision {
    let Some(id) = identity else {
        return UploadDecision::Unauthenticated;
    };
    if guard::require_moderator_or_admin(id).is_err() {
        return UploadDecision::Forbidden;
    }
    if !is_valid_sf2 {
        return UploadDecision::InvalidBody;
    }
    UploadDecision::Accept
}

/// Admin upload of a font: validate + authorize, then store the object and record the
/// catalog row (object first, so a failure never leaves a listed font without bytes).
async fn upload(
    State(s): State<SoundfontState>,
    Path(id): Path<String>,
    Query(meta): Query<UploadMeta>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let identity = s.auth.identify_admin(&headers);
    match decide_upload(identity.as_ref(), is_valid_soundfont(&body)) {
        UploadDecision::Unauthenticated => return status(StatusCode::UNAUTHORIZED),
        UploadDecision::Forbidden => return status(StatusCode::FORBIDDEN),
        UploadDecision::InvalidBody => return status(StatusCode::UNPROCESSABLE_ENTITY),
        UploadDecision::Accept => {}
    }
    if meta.label.trim().is_empty() || meta.license.trim().is_empty() {
        return status(StatusCode::BAD_REQUEST);
    }
    let (Some(store), Some(repo)) = (s.store.as_ref(), s.repo.as_ref()) else {
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    // Refuse a duplicate id rather than silently overwriting an existing font.
    match repo.lookup(&id).await {
        Ok(Some(_)) => return status(StatusCode::CONFLICT),
        Ok(None) => {}
        Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
    }

    let object_key = format!("{id}.sf2");
    let size = body.len() as i64;
    // Object first…
    if store.put(&object_key, body.to_vec()).await.is_err() {
        return status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    // …then the row. If the row write fails, best-effort remove the just-stored object
    // so we don't leave a referenced-but-unlisted blob.
    let attribution = (!meta.attribution.trim().is_empty()).then(|| meta.attribution.clone());
    let instrument = if meta.instrument.trim().is_empty() {
        "piano".to_string()
    } else {
        meta.instrument.trim().to_string()
    };
    let entry = FontEntry {
        id,
        label: meta.label,
        object_key: object_key.clone(),
        instrument,
        license: meta.license,
        attribution,
        size_bytes: Some(size),
    };
    if repo.insert(&entry).await.is_err() {
        let _ = store.delete(&object_key).await;
        return status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    status(StatusCode::CREATED)
}

async fn serve(
    State(s): State<SoundfontState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user = s.auth.identify(&headers);
    // Reject unauthenticated callers before any catalog read, so a refusal never
    // depends on (nor reveals) whether the font exists.
    if user.is_none() {
        return status(StatusCode::UNAUTHORIZED);
    }
    let Some(repo) = s.repo.as_ref() else {
        // Feature unconfigured (no music DB / catalog) — the route is disabled.
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    let font = match repo.lookup(&id).await {
        Ok(f) => f,
        Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let key = match decide(user.as_deref(), font.as_ref()) {
        Decision::Unauthenticated => return status(StatusCode::UNAUTHORIZED),
        Decision::NotFound => return status(StatusCode::NOT_FOUND),
        Decision::Serve(key) => key,
    };
    let Some(store) = s.store.as_ref() else {
        // Feature unconfigured (no SoundFont bucket) — the route is disabled.
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    stream(store.as_ref(), &key, &headers).await
}

/// Stream the object (range-aware). A `Range` request yields `206 Partial Content`
/// with `Content-Range`; a plain request yields the full `200`. A missing object is
/// `404`, a backend fault `500`.
async fn stream(store: &dyn ObjectStorage, key: &str, headers: &HeaderMap) -> Response {
    match parse_range(headers) {
        Some(range) => {
            let total = match store.size(key).await {
                Ok(n) => n,
                Err(StorageError::NotFound(_)) => return status(StatusCode::NOT_FOUND),
                Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
            };
            if total == 0 || range.start >= total {
                // Unsatisfiable range.
                return (
                    StatusCode::RANGE_NOT_SATISFIABLE,
                    [(header::CONTENT_RANGE, format!("bytes */{total}"))],
                )
                    .into_response();
            }
            let end = range.end.unwrap_or(total - 1).min(total - 1);
            let bytes = match store
                .get_range(key, range.start as usize..(end as usize + 1))
                .await
            {
                Ok(b) => b,
                Err(StorageError::NotFound(_)) => return status(StatusCode::NOT_FOUND),
                Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
            };
            (
                StatusCode::PARTIAL_CONTENT,
                [
                    (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                    (header::ACCEPT_RANGES, "bytes".to_string()),
                    (
                        header::CONTENT_RANGE,
                        format!("bytes {}-{end}/{total}", range.start),
                    ),
                    (header::CONTENT_LENGTH, bytes.len().to_string()),
                ],
                Body::from(bytes),
            )
                .into_response()
        }
        None => match store.get(key).await {
            Ok(bytes) => (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                    (header::ACCEPT_RANGES, "bytes".to_string()),
                    (header::CONTENT_LENGTH, bytes.len().to_string()),
                ],
                Body::from(bytes),
            )
                .into_response(),
            Err(StorageError::NotFound(_)) => status(StatusCode::NOT_FOUND),
            Err(_) => status(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

fn status(code: StatusCode) -> Response {
    code.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use cymbra_storage::FakeStore;
    use tower::ServiceExt;

    // --- Pure logic ------------------------------------------------------

    /// A test [`FontEntry`] (owned fields, as the DB repo yields).
    fn entry(id: &str, object_key: &str, license: &str) -> FontEntry {
        FontEntry {
            id: id.into(),
            label: id.into(),
            object_key: object_key.into(),
            instrument: "piano".into(),
            license: license.into(),
            attribution: None,
            size_bytes: None,
        }
    }

    fn upright() -> FontEntry {
        entry("upright-piano-kw", "UprightPianoKW-20220221.sf2", "CC0-1.0")
    }

    #[test]
    fn decide_requires_auth_then_the_font_to_exist() {
        let font = upright();
        assert_eq!(decide(None, Some(&font)), Decision::Unauthenticated);
        assert_eq!(decide(Some("u"), None), Decision::NotFound);
        assert_eq!(
            decide(Some("u"), Some(&font)),
            Decision::Serve("UprightPianoKW-20220221.sf2".to_string())
        );
    }

    #[test]
    fn parse_range_variants() {
        let mk = |v: &str| {
            let mut h = HeaderMap::new();
            h.insert(header::RANGE, v.parse().unwrap());
            h
        };
        assert_eq!(
            parse_range(&mk("bytes=0-99")),
            Some(ByteRange {
                start: 0,
                end: Some(99)
            })
        );
        assert_eq!(
            parse_range(&mk("bytes=100-")),
            Some(ByteRange {
                start: 100,
                end: None
            })
        );
        assert_eq!(parse_range(&mk("bytes=5-2")), None); // end < start
        assert_eq!(parse_range(&mk("bytes=0-10,20-30")), None); // multi-range unsupported
        assert_eq!(parse_range(&HeaderMap::new()), None);
    }

    // --- Handler via oneshot ---------------------------------------------

    struct FixedAuth(Option<&'static str>);
    impl SoundfontAuth for FixedAuth {
        fn identify(&self, _h: &HeaderMap) -> Option<String> {
            self.0.map(str::to_string)
        }
    }

    /// Router with a catalog seeded with the CC0 default (the common case).
    async fn app(store: Option<Arc<dyn ObjectStorage>>, user: Option<&'static str>) -> Router {
        app_with_repo(
            store,
            Some(Arc::new(cymbra_music::FakeSoundFontRepo::with(vec![
                upright(),
            ]))),
            user,
        )
    }

    fn app_with_repo(
        store: Option<Arc<dyn ObjectStorage>>,
        repo: Option<Arc<dyn SoundFontRepo>>,
        user: Option<&'static str>,
    ) -> Router {
        soundfont_router(
            SoundfontState {
                store,
                repo,
                auth: Arc::new(FixedAuth(user)),
            },
            vec!["https://bo.cymbra.app".to_string()],
        )
    }

    #[tokio::test]
    async fn unconfigured_repo_is_503() {
        // No catalog (music DB absent) ⇒ the route is disabled even with a store.
        let r = app_with_repo(Some(seeded_store().await), None, Some("u"));
        let resp = r
            .oneshot(get("/soundfonts/upright-piano-kw"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    fn get(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    async fn seeded_store() -> Arc<dyn ObjectStorage> {
        let s = FakeStore::default();
        s.put("UprightPianoKW-20220221.sf2", b"SF2-BYTES!!".to_vec())
            .await
            .unwrap();
        Arc::new(s)
    }

    #[tokio::test]
    async fn unauthenticated_is_401() {
        let r = app(Some(seeded_store().await), None).await;
        let resp = r
            .oneshot(get("/soundfonts/upright-piano-kw"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn unknown_id_is_404() {
        let r = app(Some(seeded_store().await), Some("u")).await;
        let resp = r.oneshot(get("/soundfonts/nope")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unconfigured_store_is_503() {
        let r = app(None, Some("u")).await;
        let resp = r
            .oneshot(get("/soundfonts/upright-piano-kw"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn authorized_full_get_streams_bytes() {
        let r = app(Some(seeded_store().await), Some("u")).await;
        let resp = r
            .oneshot(get("/soundfonts/upright-piano-kw"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::ACCEPT_RANGES).unwrap(), "bytes");
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"SF2-BYTES!!");
    }

    #[tokio::test]
    async fn range_request_serves_206_partial() {
        let r = app(Some(seeded_store().await), Some("u")).await;
        let req = Request::builder()
            .uri("/soundfonts/upright-piano-kw")
            .header(header::RANGE, "bytes=0-3")
            .body(Body::empty())
            .unwrap();
        let resp = r.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            resp.headers().get(header::CONTENT_RANGE).unwrap(),
            "bytes 0-3/11"
        );
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"SF2-"); // first 4 bytes
    }

    #[tokio::test]
    async fn unsatisfiable_range_is_416() {
        let r = app(Some(seeded_store().await), Some("u")).await;
        let req = Request::builder()
            .uri("/soundfonts/upright-piano-kw")
            .header(header::RANGE, "bytes=999-")
            .body(Body::empty())
            .unwrap();
        let resp = r.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    }

    // --- Upload (admin) --------------------------------------------------

    fn ident(user: &str, roles: &[&str]) -> AuthIdentity {
        AuthIdentity {
            user_id: user.into(),
            audience: "music".into(),
            roles: roles.iter().map(|r| (*r).into()).collect(),
            ..Default::default()
        }
    }

    /// Auth double returning a full identity for the admin upload route.
    struct FixedAdminAuth(Option<AuthIdentity>);
    impl SoundfontAuth for FixedAdminAuth {
        fn identify(&self, _h: &HeaderMap) -> Option<String> {
            self.0.as_ref().map(|i| i.user_id.clone())
        }
        fn identify_admin(&self, _h: &HeaderMap) -> Option<AuthIdentity> {
            self.0.clone()
        }
    }

    fn upload_app(store: Arc<dyn ObjectStorage>, identity: Option<AuthIdentity>) -> Router {
        soundfont_router(
            SoundfontState {
                store: Some(store),
                repo: Some(Arc::new(cymbra_music::FakeSoundFontRepo::default())),
                auth: Arc::new(FixedAdminAuth(identity)),
            },
            vec!["https://bo.cymbra.app".to_string()],
        )
    }

    /// A minimal valid `.sf2` preamble: `RIFF <size> sfbk`.
    fn sf2_bytes() -> Vec<u8> {
        let mut b = b"RIFF".to_vec();
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(b"sfbk");
        b
    }

    fn post(uri: &str, body: Vec<u8>) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Body::from(body))
            .unwrap()
    }

    const UPLOAD_URI: &str = "/soundfonts/ydp-grand?label=YDP%20Grand&license=CC-BY%203.0";

    #[test]
    fn decide_upload_orders_auth_then_admin_then_body() {
        assert_eq!(decide_upload(None, true), UploadDecision::Unauthenticated);
        let user = ident("u", &["user"]);
        assert_eq!(decide_upload(Some(&user), true), UploadDecision::Forbidden);
        let admin = ident("a", &["admin"]);
        assert_eq!(
            decide_upload(Some(&admin), false),
            UploadDecision::InvalidBody
        );
        assert_eq!(decide_upload(Some(&admin), true), UploadDecision::Accept);
    }

    #[tokio::test]
    async fn upload_unauthenticated_is_401() {
        let r = upload_app(Arc::new(FakeStore::default()), None);
        let resp = r.oneshot(post(UPLOAD_URI, sf2_bytes())).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upload_non_admin_is_403() {
        let r = upload_app(Arc::new(FakeStore::default()), Some(ident("u", &["user"])));
        let resp = r.oneshot(post(UPLOAD_URI, sf2_bytes())).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn upload_invalid_body_is_422() {
        let r = upload_app(Arc::new(FakeStore::default()), Some(ident("a", &["admin"])));
        let resp = r
            .oneshot(post(UPLOAD_URI, b"not a soundfont".to_vec()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn upload_admin_stores_object_and_row() {
        let store = Arc::new(FakeStore::default());
        let repo = Arc::new(cymbra_music::FakeSoundFontRepo::default());
        let r = soundfont_router(
            SoundfontState {
                store: Some(store.clone()),
                repo: Some(repo.clone()),
                auth: Arc::new(FixedAdminAuth(Some(ident("a", &["admin"])))),
            },
            vec!["https://bo.cymbra.app".to_string()],
        );
        let resp = r.oneshot(post(UPLOAD_URI, sf2_bytes())).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        // Object stored under {id}.sf2 and a catalog row recorded.
        assert!(store.size("ydp-grand.sf2").await.is_ok());
        let row = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(row.label, "YDP Grand");
        assert_eq!(row.object_key, "ydp-grand.sf2");
        assert_eq!(row.license, "CC-BY 3.0");
        // Instrument defaults to piano when not supplied.
        assert_eq!(row.instrument, "piano");
    }

    #[tokio::test]
    async fn upload_duplicate_id_is_409() {
        let store = Arc::new(FakeStore::default());
        let repo: Arc<dyn SoundFontRepo> =
            Arc::new(cymbra_music::FakeSoundFontRepo::with(vec![FontEntry {
                id: "ydp-grand".into(),
                label: "existing".into(),
                object_key: "ydp-grand.sf2".into(),
                instrument: "piano".into(),
                license: "CC-BY 3.0".into(),
                attribution: None,
                size_bytes: None,
            }]));
        let r = soundfont_router(
            SoundfontState {
                store: Some(store),
                repo: Some(repo),
                auth: Arc::new(FixedAdminAuth(Some(ident("a", &["admin"])))),
            },
            vec!["https://bo.cymbra.app".to_string()],
        );
        let resp = r.oneshot(post(UPLOAD_URI, sf2_bytes())).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }
}
