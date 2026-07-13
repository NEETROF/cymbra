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

//! HTTP access for the web-crawl adapters.
//!
//! The [`Fetcher`] trait abstracts "GET this URL" so adapters are unit-tested
//! against fixture content ([`MapFetcher`]) with no network. [`HttpFetcher`] is
//! the real implementation: a descriptive User-Agent, gzip, a polite per-request
//! delay, exponential back-off on transient failures ([`crate::politeness`]),
//! and robots.txt enforcement per host. The real client is network glue and is
//! excluded from unit tests; the parsing that adapters build on top is not.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::politeness::{Backoff, retry_async};
use crate::robots::RobotsPolicy;

/// "GET this URL" as text or bytes. Adapters depend on this, not on reqwest.
#[async_trait]
pub trait Fetcher: Send + Sync {
    async fn get_text(&self, url: &str) -> Result<String>;
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;
}

/// A transient/permanent split so back-off only retries the transient class.
#[derive(Debug, thiserror::Error)]
enum FetchError {
    #[error("transient: {0}")]
    Transient(String),
    #[error("permanent: {0}")]
    Permanent(String),
}

impl FetchError {
    fn is_transient(&self) -> bool {
        matches!(self, FetchError::Transient(_))
    }
}

/// The real reqwest-backed fetcher: UA + gzip + delay + back-off + robots.
pub struct HttpFetcher {
    client: reqwest::Client,
    user_agent: String,
    backoff: Backoff,
    delay: Duration,
    /// Per-host robots policy cache (fetched lazily). `None` = allow-all (no /
    /// unreadable / unparseable robots.txt).
    robots: Mutex<HashMap<String, Option<RobotsPolicy>>>,
}

impl HttpFetcher {
    /// Builds a fetcher with a descriptive User-Agent and a per-request delay.
    pub fn new(user_agent: impl Into<String>, delay: Duration) -> Result<Self> {
        let user_agent = user_agent.into();
        let client = reqwest::Client::builder()
            .user_agent(&user_agent)
            .gzip(true)
            .timeout(Duration::from_secs(30))
            .build()
            .context("building HTTP client")?;
        Ok(Self {
            client,
            user_agent,
            backoff: Backoff::default(),
            delay,
            robots: Mutex::new(HashMap::new()),
        })
    }

    /// Ensures `url` is allowed by its host's robots.txt (fetched + cached once).
    async fn ensure_allowed(&self, url: &str) -> Result<(), FetchError> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|e| FetchError::Permanent(format!("bad url {url}: {e}")))?;
        let host = match parsed.host_str() {
            Some(h) => h.to_string(),
            None => return Ok(()), // no host (e.g. file://) → nothing to check
        };
        let mut cache = self.robots.lock().await;
        if !cache.contains_key(&host) {
            let robots_url = format!("{}://{}/robots.txt", parsed.scheme(), host);
            // A missing / unreadable / unparseable robots.txt ⇒ allow-all (None).
            let policy = match self.client.get(&robots_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let body = resp.bytes().await.unwrap_or_default();
                    RobotsPolicy::parse(&self.user_agent, &body).ok()
                }
                _ => None,
            };
            cache.insert(host.clone(), policy);
        }
        // Absent policy or unknown host ⇒ allowed.
        let allowed = cache
            .get(&host)
            .and_then(|opt| opt.as_ref())
            .map(|p| p.allowed(url))
            .unwrap_or(true);
        if allowed {
            Ok(())
        } else {
            Err(FetchError::Permanent(format!("robots.txt disallows {url}")))
        }
    }

    /// One polite GET attempt returning the raw response (delay applied first).
    async fn get_once(&self, url: &str) -> Result<reqwest::Response, FetchError> {
        tokio::time::sleep(self.delay).await;
        let resp = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() || e.is_connect() {
                FetchError::Transient(e.to_string())
            } else {
                FetchError::Permanent(e.to_string())
            }
        })?;
        let status = resp.status();
        if status.as_u16() == 429 || status.is_server_error() {
            return Err(FetchError::Transient(format!("HTTP {status}")));
        }
        if !status.is_success() {
            return Err(FetchError::Permanent(format!("HTTP {status}")));
        }
        Ok(resp)
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn get_text(&self, url: &str) -> Result<String> {
        self.ensure_allowed(url).await.map_err(|e| anyhow!("{e}"))?;
        let resp = retry_async(&self.backoff, FetchError::is_transient, || {
            self.get_once(url)
        })
        .await
        .map_err(|e| anyhow!("GET {url}: {e}"))?;
        resp.text()
            .await
            .with_context(|| format!("reading body of {url}"))
    }

    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        self.ensure_allowed(url).await.map_err(|e| anyhow!("{e}"))?;
        let resp = retry_async(&self.backoff, FetchError::is_transient, || {
            self.get_once(url)
        })
        .await
        .map_err(|e| anyhow!("GET {url}: {e}"))?;
        Ok(resp
            .bytes()
            .await
            .with_context(|| format!("reading body of {url}"))?
            .to_vec())
    }
}

#[cfg(test)]
pub(crate) mod test_fetcher {
    use std::collections::HashMap;

    use super::*;

    /// A fixture-backed fetcher: serves in-memory content by exact URL, so
    /// adapter parsing is tested with no network.
    #[derive(Default)]
    pub struct MapFetcher {
        pub pages: HashMap<String, String>,
        pub blobs: HashMap<String, Vec<u8>>,
    }

    impl MapFetcher {
        pub fn with_page(mut self, url: &str, body: &str) -> Self {
            self.pages.insert(url.to_string(), body.to_string());
            self
        }
        pub fn with_blob(mut self, url: &str, body: &[u8]) -> Self {
            self.blobs.insert(url.to_string(), body.to_vec());
            self
        }
    }

    #[async_trait]
    impl Fetcher for MapFetcher {
        async fn get_text(&self, url: &str) -> Result<String> {
            self.pages
                .get(url)
                .cloned()
                .ok_or_else(|| anyhow!("no fixture page for {url}"))
        }
        async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
            self.blobs
                .get(url)
                .cloned()
                .or_else(|| self.pages.get(url).map(|s| s.clone().into_bytes()))
                .ok_or_else(|| anyhow!("no fixture blob for {url}"))
        }
    }

    #[tokio::test]
    async fn map_fetcher_serves_fixtures() {
        let f = MapFetcher::default()
            .with_page("https://h/index", "<html>hi</html>")
            .with_blob("https://h/score.musicxml", b"<score-partwise/>");
        assert_eq!(
            f.get_text("https://h/index").await.unwrap(),
            "<html>hi</html>"
        );
        assert_eq!(
            f.get_bytes("https://h/score.musicxml").await.unwrap(),
            b"<score-partwise/>"
        );
        assert!(f.get_text("https://h/missing").await.is_err());
    }
}
