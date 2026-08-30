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

//! The desktop update feed HTTP surface (change: add-desktop-auto-update,
//! tasks 2.3–2.5, design D1/D3).
//!
//! - `GET  /updates/desktop?product=&channel=` — **public and anonymous**. The
//!   same bytes for every caller: no current-version parameter, no install id,
//!   no account. That is what makes it one cacheable document per
//!   product/channel and what makes the update check unusable for counting or
//!   tracking installs.
//! - `POST /updates/desktop` — credential-gated ingest, called by the release
//!   workflow. It **re-verifies the CI signature before storing**, so a stolen
//!   ingest credential still cannot inject an unsigned or foreign-key manifest:
//!   the credential decides *whether* something is stored, never *what*.
//!
//! The product/channel/version a row is stored under are read from the **signed
//! manifest**, never from the request, so ingest cannot mislabel a release into
//! another product's feed.
//!
//! ⚠️ Ops: `/updates/*` must be in the Caddy `@http` path matcher
//! (`backend/deploy/Caddyfile`). A path missing from that allow-list falls
//! through to tonic and answers `200` with an empty `grpc-status: 12` — the
//! client then sees a malformed response and the update check fails invisibly.

use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use cymbra_update_manifest::{Envelope, TrustedKeys, VerifyError};
use cymbra_updates::{Release, ReleaseRepo, version_order};
use serde::Deserialize;

/// How long a feed response may be cached. The kill-switch (`rollout_percent = 0`)
/// therefore takes effect within this window, not instantly — the trade for a
/// fully anonymous, CDN-friendly endpoint.
const FEED_MAX_AGE_SECS: u32 = 300;

/// Router state. Every field is optional so an unconfigured deployment stays
/// inert rather than half-working: no repo ⇒ the feed answers `204` (nothing is
/// ever offered), no trusted keys or no secret ⇒ ingest is closed.
#[derive(Clone)]
pub struct UpdatesState {
    pub repo: Option<Arc<dyn ReleaseRepo>>,
    pub trusted_keys: Arc<TrustedKeys>,
    pub ingest_secret: Option<Arc<String>>,
}

/// The desktop update feed router, ready to `.merge()` into the HTTP server.
///
/// No CORS layer: the consumers are the desktop apps, not a browser.
pub fn updates_router(state: UpdatesState) -> Router {
    Router::new()
        .route("/updates/desktop", get(serve_feed).post(ingest_release))
        .with_state(state)
}

/// `?product=music&channel=stable` — the only request-scoped input, and both
/// default so a bare `GET /updates/desktop` is the Music stable feed.
#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_product")]
    pub product: String,
    #[serde(default = "default_channel")]
    pub channel: String,
}

fn default_product() -> String {
    "music".to_string()
}

fn default_channel() -> String {
    "stable".to_string()
}

/// `GET /updates/desktop` — the servable release, or `204` when there is none.
async fn serve_feed(State(s): State<UpdatesState>, Query(q): Query<FeedQuery>) -> Response {
    let Some(repo) = s.repo.as_ref() else {
        // Unconfigured is indistinguishable from "nothing to offer", on purpose:
        // the client's no-op path is identical either way.
        return no_content();
    };
    match repo.servable(&q.product, &q.channel).await {
        Ok(Some(release)) => feed_response(&release),
        // An unknown product or channel is not an error — it simply has nothing
        // to offer, and saying so would make the endpoint an existence oracle.
        Ok(None) => no_content(),
        Err(e) => {
            tracing::warn!(error = %e, product = %q.product, channel = %q.channel, "update feed read failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn no_content() -> Response {
    (
        StatusCode::NO_CONTENT,
        [(header::CACHE_CONTROL, cache_control())],
    )
        .into_response()
}

fn cache_control() -> String {
    format!("public, max-age={FEED_MAX_AGE_SECS}")
}

fn feed_response(release: &Release) -> Response {
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, cache_control())],
        axum::Json(release.to_envelope()),
    )
        .into_response()
}

/// `POST /updates/desktop` — the release workflow publishing a signed envelope.
async fn ingest_release(
    State(s): State<UpdatesState>,
    headers: HeaderMap,
    body: String,
) -> StatusCode {
    let Some(expected) = s.ingest_secret.as_ref() else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    if !credential_matches(&headers, expected) {
        return StatusCode::UNAUTHORIZED;
    }
    let Some(repo) = s.repo.as_ref() else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    let release = match release_from_envelope(&body, &s.trusted_keys) {
        Ok(r) => r,
        Err(status) => return status,
    };
    match repo.upsert(&release).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!(error = %e, version = %release.version, "update ingest write failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Constant-time bearer check. Constant-time so the endpoint does not leak the
/// secret one byte at a time to a caller that can measure it.
fn credential_matches(headers: &HeaderMap, expected: &str) -> bool {
    let presented = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    constant_time_eq(presented.as_bytes(), expected.as_bytes())
}

/// Length is not secret (and an early length return would itself be a timing
/// signal only about the length), so compare the bytes without short-circuiting.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

/// Verify an ingest payload and turn it into the row to store.
///
/// Pure and host-testable (task 2.8): the ordering is the whole security
/// property — signature first, and every field the row is keyed by comes from
/// the **verified** manifest, never from the request.
pub fn release_from_envelope(body: &str, trusted: &TrustedKeys) -> Result<Release, StatusCode> {
    let envelope: Envelope = serde_json::from_str(body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let manifest = cymbra_update_manifest::verify(&envelope, trusted).map_err(|e| match e {
        // A key we do not trust is an authorization problem, not a syntax one:
        // the payload is well-formed, it is just not ours to publish.
        VerifyError::UnknownKeyId => StatusCode::FORBIDDEN,
        VerifyError::BadSignature => StatusCode::FORBIDDEN,
        _ => StatusCode::BAD_REQUEST,
    })?;
    let order = version_order(&manifest.version).ok_or(StatusCode::BAD_REQUEST)?;
    if manifest.targets.is_empty() {
        // A release with nothing to download would be offered and then fail on
        // every client; refuse it at the door.
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Release {
        product: manifest.product,
        channel: manifest.channel,
        version: manifest.version,
        version_order: order,
        manifest: envelope.manifest,
        signature: envelope.signature,
        key_id: envelope.key_id,
        rollout_percent: envelope.rollout_percent.min(100) as i16,
        paused: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD as B64;
    use cymbra_update_manifest::{Manifest, SCHEMA_VERSION, Target, sign};
    use cymbra_updates::MockReleaseRepo;
    use ed25519_dalek::SigningKey;
    use std::collections::BTreeMap;
    use tower::ServiceExt as _;

    const SECRET: [u8; 32] = [3u8; 32];
    const KEY_ID: &str = "test-a";

    fn key() -> SigningKey {
        SigningKey::from_bytes(&SECRET)
    }

    fn trusted() -> Arc<TrustedKeys> {
        Arc::new(TrustedKeys::from([(
            KEY_ID.to_string(),
            key().verifying_key(),
        )]))
    }

    fn manifest(version: &str) -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: "music".into(),
            channel: "stable".into(),
            version: version.into(),
            released_at: "2026-08-21T10:00:00Z".into(),
            min_supported_version: None,
            notes_url: None,
            targets: BTreeMap::from([(
                "windows-x64".to_string(),
                Target {
                    kind: "inno-setup".into(),
                    url: "https://example.invalid/setup.exe".into(),
                    size: 1,
                    sha256: "a".repeat(64),
                },
            )]),
        }
    }

    fn envelope_json(m: &Manifest, rollout: u8) -> String {
        let bytes = serde_json::to_vec(m).unwrap();
        serde_json::to_string(&sign(&bytes, &key(), KEY_ID, rollout)).unwrap()
    }

    fn release(version: &str) -> Release {
        Release {
            product: "music".into(),
            channel: "stable".into(),
            version: version.into(),
            version_order: version_order(version).unwrap(),
            manifest: B64.encode(serde_json::to_vec(&manifest(version)).unwrap()),
            signature: B64.encode([1u8; 64]),
            key_id: KEY_ID.into(),
            rollout_percent: 25,
            paused: false,
        }
    }

    fn state(repo: Option<Arc<dyn ReleaseRepo>>, secret: Option<&str>) -> UpdatesState {
        UpdatesState {
            repo,
            trusted_keys: trusted(),
            ingest_secret: secret.map(|s| Arc::new(s.to_string())),
        }
    }

    async fn call(state: UpdatesState, req: Request<Body>) -> (StatusCode, HeaderMap, String) {
        let res = updates_router(state).oneshot(req).await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = axum::body::to_bytes(res.into_body(), 1 << 20)
            .await
            .unwrap();
        (status, headers, String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    fn post(body: &str, bearer: Option<&str>) -> Request<Body> {
        let mut b = Request::builder().method("POST").uri("/updates/desktop");
        if let Some(t) = bearer {
            b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        b.body(Body::from(body.to_string())).unwrap()
    }

    // --- the public feed ---------------------------------------------------

    #[tokio::test]
    async fn serves_the_stored_envelope_with_a_cache_header() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_servable()
            .withf(|p, c| p == "music" && c == "stable")
            .returning(|_, _| Ok(Some(release("1.25.0+34"))));
        let (status, headers, body) = call(
            state(Some(Arc::new(repo)), None),
            get("/updates/desktop?product=music&channel=stable"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers.get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=300"
        );
        let env: Envelope = serde_json::from_str(&body).unwrap();
        assert_eq!(env.key_id, KEY_ID);
        assert_eq!(env.rollout_percent, 25);
        // The served bytes are the stored bytes: what CI signed is what ships.
        assert_eq!(env.manifest, release("1.25.0+34").manifest);
        assert_eq!(env.signature, release("1.25.0+34").signature);
    }

    #[tokio::test]
    async fn product_and_channel_default_to_music_stable() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_servable()
            .withf(|p, c| p == "music" && c == "stable")
            .returning(|_, _| Ok(None));
        let (status, _, _) = call(state(Some(Arc::new(repo)), None), get("/updates/desktop")).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn nothing_servable_answers_204() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_servable().returning(|_, _| Ok(None));
        let (status, headers, _) = call(
            state(Some(Arc::new(repo)), None),
            get("/updates/desktop?product=music&channel=stable"),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(headers.get(header::CACHE_CONTROL).is_some());
    }

    #[tokio::test]
    async fn an_unknown_product_answers_204_not_an_error() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_servable()
            .withf(|p, _| p == "nope")
            .returning(|_, _| Ok(None));
        let (status, _, _) = call(
            state(Some(Arc::new(repo)), None),
            get("/updates/desktop?product=nope&channel=stable"),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn an_unwired_feed_answers_204_rather_than_failing() {
        let (status, _, _) = call(state(None, None), get("/updates/desktop")).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn a_read_failure_is_a_500_not_a_silent_204() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_servable()
            .returning(|_, _| Err(anyhow::anyhow!("boom")));
        let (status, _, _) = call(state(Some(Arc::new(repo)), None), get("/updates/desktop")).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    // --- ingest ------------------------------------------------------------

    #[tokio::test]
    async fn ingest_stores_a_correctly_signed_release() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_upsert()
            .withf(|r| {
                r.product == "music"
                    && r.channel == "stable"
                    && r.version == "1.25.0+34"
                    && r.rollout_percent == 0
                    && !r.paused
            })
            .returning(|_| Ok(()));
        let body = envelope_json(&manifest("1.25.0+34"), 0);
        let (status, _, _) = call(
            state(Some(Arc::new(repo)), Some("s3cr3t")),
            post(&body, Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn ingest_refuses_a_wrong_credential_before_looking_at_the_body() {
        // No repo expectation is set: reaching the store would fail the test.
        let body = envelope_json(&manifest("1.25.0+34"), 0);
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), Some("s3cr3t")),
            post(&body, Some("wrong")),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ingest_refuses_a_missing_credential() {
        let body = envelope_json(&manifest("1.25.0+34"), 0);
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), Some("s3cr3t")),
            post(&body, None),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ingest_is_closed_when_no_secret_is_configured() {
        let body = envelope_json(&manifest("1.25.0+34"), 0);
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), None),
            post(&body, Some("anything")),
        )
        .await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn ingest_refuses_a_tampered_manifest_even_with_a_valid_credential() {
        let body = envelope_json(&manifest("1.25.0+34"), 0);
        let mut env: Envelope = serde_json::from_str(&body).unwrap();
        let swapped = String::from_utf8(B64.decode(&env.manifest).unwrap())
            .unwrap()
            .replace("1.25.0+34", "9.99.9+99");
        env.manifest = B64.encode(swapped.as_bytes());
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), Some("s3cr3t")),
            post(&serde_json::to_string(&env).unwrap(), Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ingest_refuses_a_foreign_signing_key() {
        let other = SigningKey::from_bytes(&[8u8; 32]);
        let bytes = serde_json::to_vec(&manifest("1.25.0+34")).unwrap();
        let env = sign(&bytes, &other, KEY_ID, 0);
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), Some("s3cr3t")),
            post(&serde_json::to_string(&env).unwrap(), Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ingest_refuses_an_unknown_key_id() {
        let bytes = serde_json::to_vec(&manifest("1.25.0+34")).unwrap();
        let env = sign(&bytes, &key(), "rotated-out", 0);
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), Some("s3cr3t")),
            post(&serde_json::to_string(&env).unwrap(), Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ingest_refuses_a_body_that_is_not_an_envelope() {
        let (status, _, _) = call(
            state(Some(Arc::new(MockReleaseRepo::new())), Some("s3cr3t")),
            post("{\"nope\":true}", Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn re_ingesting_the_same_version_replaces_it() {
        // The upsert key is (product, channel, version); raising a rollout is a
        // re-ingest of the same version with a new percentage.
        let mut repo = MockReleaseRepo::new();
        repo.expect_upsert()
            .times(1)
            .withf(|r| r.version == "1.25.0+34" && r.rollout_percent == 50)
            .returning(|_| Ok(()));
        let body = envelope_json(&manifest("1.25.0+34"), 50);
        let (status, _, _) = call(
            state(Some(Arc::new(repo)), Some("s3cr3t")),
            post(&body, Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn a_write_failure_is_a_500() {
        let mut repo = MockReleaseRepo::new();
        repo.expect_upsert()
            .returning(|_| Err(anyhow::anyhow!("boom")));
        let body = envelope_json(&manifest("1.25.0+34"), 0);
        let (status, _, _) = call(
            state(Some(Arc::new(repo)), Some("s3cr3t")),
            post(&body, Some("s3cr3t")),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    // --- the pure ingest decision -----------------------------------------

    #[test]
    fn the_row_is_keyed_by_the_signed_manifest_not_the_request() {
        let body = envelope_json(&manifest("1.25.0+34"), 25);
        let row = release_from_envelope(&body, &trusted()).unwrap();
        assert_eq!(row.product, "music");
        assert_eq!(row.channel, "stable");
        assert_eq!(row.version, "1.25.0+34");
        assert_eq!(row.version_order, version_order("1.25.0+34").unwrap());
        assert!(!row.paused);
    }

    #[test]
    fn an_unsortable_version_is_refused() {
        let mut m = manifest("not-a-version");
        m.version = "not-a-version".into();
        let body = envelope_json(&m, 0);
        assert_eq!(
            release_from_envelope(&body, &trusted()),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn a_release_with_no_targets_is_refused() {
        let mut m = manifest("1.25.0+34");
        m.targets.clear();
        let body = envelope_json(&m, 0);
        assert_eq!(
            release_from_envelope(&body, &trusted()),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn a_future_schema_is_refused() {
        let mut m = manifest("1.25.0+34");
        m.schema = SCHEMA_VERSION + 1;
        let body = envelope_json(&m, 0);
        assert_eq!(
            release_from_envelope(&body, &trusted()),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn constant_time_eq_still_answers_correctly() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn only_a_bearer_scheme_is_accepted() {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, "Basic s3cr3t".parse().unwrap());
        assert!(!credential_matches(&h, "s3cr3t"));
        h.insert(header::AUTHORIZATION, "Bearer s3cr3t".parse().unwrap());
        assert!(credential_matches(&h, "s3cr3t"));
    }
}
