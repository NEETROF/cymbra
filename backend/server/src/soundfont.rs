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

//! SoundFont delivery (change: add-soundfont-delivery).
//!
//! An authenticated HTTP route `GET /soundfonts/{id}` that streams a SoundFont's
//! bytes from the **private** SoundFont object store, gated by a per-font entitlement
//! check. Free (CC0/CC-BY) fonts are granted to any signed-in caller; paid fonts (none
//! yet) are gated by an [`Entitlements`] source — the seam that makes adding paid fonts
//! later a data change, not a redesign.
//!
//! The **decision** logic (auth present? font known? entitled?) and range parsing are
//! pure and host-tested; auth (JWT) and the object-store IO sit behind injectable
//! seams ([`SoundfontAuth`], [`ObjectStorage`]) so the whole handler is testable with
//! fakes — no JWT minting, no network.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::http::{HeaderValue, Method};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use cymbra_platform::token;
use cymbra_storage::{ObjectStorage, StorageError};
use jsonwebtoken::DecodingKey;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Access tier of a font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Redistributable (CC0/CC-BY) — any authenticated caller may fetch it.
    Free,
    /// Requires an entitlement (a future purchase). None exist yet.
    Paid,
}

/// A catalog entry: the client-facing id, the storage key, the access tier, and the
/// licence/attribution (recorded for CC-BY redistribution).
#[derive(Debug, Clone)]
pub struct FontEntry {
    pub id: &'static str,
    pub object_key: &'static str,
    pub tier: Tier,
    pub license: &'static str,
    pub attribution: Option<&'static str>,
}

/// The server-owned SoundFont catalog. v1: the free CC0 default (the app's bundled
/// piano). Paid grands are added here (with `Tier::Paid`) once the purchase system
/// exists.
pub fn catalog() -> &'static [FontEntry] {
    const CATALOG: &[FontEntry] = &[FontEntry {
        id: "upright-piano-kw",
        object_key: "UprightPianoKW-20220221.sf2",
        tier: Tier::Free,
        license: "CC0-1.0",
        attribution: None,
    }];
    CATALOG
}

/// Resolve a client-facing id to its catalog entry.
pub fn lookup(id: &str) -> Option<&'static FontEntry> {
    catalog().iter().find(|f| f.id == id)
}

/// Source of truth for paid-font ownership. v1 has no paid fonts; the real
/// implementation (a purchase record) drops in behind this trait without touching the
/// route.
pub trait Entitlements: Send + Sync {
    /// Whether `user_id` is entitled to the paid font `font_id`.
    fn owns(&self, user_id: &str, font_id: &str) -> bool;
}

/// v1 entitlement source: nobody owns any paid font yet.
pub struct NoPaidEntitlements;

impl Entitlements for NoPaidEntitlements {
    fn owns(&self, _user_id: &str, _font_id: &str) -> bool {
        false
    }
}

/// Pure entitlement decision: free ⇒ allow any authenticated identity; paid ⇒ allow
/// only if the entitlement source says the identity owns it.
pub fn may_access(font: &FontEntry, user_id: &str, ent: &dyn Entitlements) -> bool {
    match font.tier {
        Tier::Free => true,
        Tier::Paid => ent.owns(user_id, font.id),
    }
}

/// The outcome of the pure request decision, before any bytes are read.
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Unauthenticated,
    NotFound,
    Forbidden,
    Serve(&'static str), // the object key to stream
}

/// Decide a delivery request purely: authenticate, resolve the font, check
/// entitlement — in that order, and **before** any storage access, so a refusal never
/// depends on whether the object exists.
pub fn decide(user_id: Option<&str>, id: &str, ent: &dyn Entitlements) -> Decision {
    let Some(user) = user_id else {
        return Decision::Unauthenticated;
    };
    let Some(font) = lookup(id) else {
        return Decision::NotFound;
    };
    if may_access(font, user, ent) {
        Decision::Serve(font.object_key)
    } else {
        Decision::Forbidden
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

/// Resolve the caller's user id from request headers, or `None` if unauthenticated.
/// Injectable so the handler is testable without minting a JWT.
pub trait SoundfontAuth: Send + Sync {
    fn identify(&self, headers: &HeaderMap) -> Option<String>;
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

impl SoundfontAuth for JwtAuth {
    fn identify(&self, headers: &HeaderMap) -> Option<String> {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))?;
        let auds: Vec<&str> = self.audiences.iter().map(String::as_str).collect();
        token::verify(token, &self.keys, &auds).ok().map(|c| c.sub)
    }
}

/// Router state: the (optional) SoundFont store, the auth seam, and the entitlement
/// source. `store: None` means the feature is unconfigured (the route reports 503).
#[derive(Clone)]
pub struct SoundfontState {
    pub store: Option<Arc<dyn ObjectStorage>>,
    pub auth: Arc<dyn SoundfontAuth>,
    pub entitlements: Arc<dyn Entitlements>,
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
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::RANGE])
        .expose_headers([
            header::CONTENT_RANGE,
            header::ACCEPT_RANGES,
            header::CONTENT_LENGTH,
        ]);
    Router::new()
        .route("/soundfonts/:id", get(serve))
        .layer(cors)
        .with_state(state)
}

async fn serve(
    State(s): State<SoundfontState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user = s.auth.identify(&headers);
    let key = match decide(user.as_deref(), &id, s.entitlements.as_ref()) {
        Decision::Unauthenticated => return status(StatusCode::UNAUTHORIZED),
        Decision::NotFound => return status(StatusCode::NOT_FOUND),
        Decision::Forbidden => return status(StatusCode::FORBIDDEN),
        Decision::Serve(key) => key,
    };
    let Some(store) = s.store.as_ref() else {
        // Feature unconfigured (no SoundFont bucket) — the route is disabled.
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    stream(store.as_ref(), key, &headers).await
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

    struct AllowAll;
    impl Entitlements for AllowAll {
        fn owns(&self, _u: &str, _f: &str) -> bool {
            true
        }
    }

    #[test]
    fn catalog_lookup_and_free_default() {
        let f = lookup("upright-piano-kw").expect("default in catalog");
        assert_eq!(f.tier, Tier::Free);
        assert_eq!(f.license, "CC0-1.0");
        assert!(lookup("nope").is_none());
    }

    #[test]
    fn entitlement_free_allows_any_paid_needs_ownership() {
        let free = FontEntry {
            id: "f",
            object_key: "f.sf2",
            tier: Tier::Free,
            license: "CC0-1.0",
            attribution: None,
        };
        let paid = FontEntry {
            id: "p",
            object_key: "p.sf2",
            tier: Tier::Paid,
            license: "proprietary",
            attribution: None,
        };
        assert!(may_access(&free, "u", &NoPaidEntitlements));
        assert!(!may_access(&paid, "u", &NoPaidEntitlements));
        assert!(may_access(&paid, "u", &AllowAll));
    }

    #[test]
    fn decide_orders_auth_then_lookup_then_entitlement() {
        assert_eq!(
            decide(None, "upright-piano-kw", &NoPaidEntitlements),
            Decision::Unauthenticated
        );
        assert_eq!(
            decide(Some("u"), "unknown", &NoPaidEntitlements),
            Decision::NotFound
        );
        assert_eq!(
            decide(Some("u"), "upright-piano-kw", &NoPaidEntitlements),
            Decision::Serve("UprightPianoKW-20220221.sf2")
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

    async fn app(store: Option<Arc<dyn ObjectStorage>>, user: Option<&'static str>) -> Router {
        soundfont_router(
            SoundfontState {
                store,
                auth: Arc::new(FixedAuth(user)),
                entitlements: Arc::new(NoPaidEntitlements),
            },
            vec!["https://bo.cymbra.app".to_string()],
        )
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
}
