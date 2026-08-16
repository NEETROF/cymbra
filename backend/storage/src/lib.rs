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

//! Object-storage port: `put` / `get` / `delete` by key.
//!
//! Two producers share one read path (design 4b): the score crawler writes its
//! corpus to **local disk** (and mirrors it to S3), while user uploads are `put`
//! to **S3** (the durable origin). Reads are **local-first with an S3 fallback**:
//! a local miss lazily pulls from S3 and warms the local copy, so a rebuilt or
//! empty box still serves everything. The same `object_key` addresses both
//! stores, so no per-source branching is needed.
//!
//! [`ObjectStorage`] is the trait the score module depends on; [`FakeStore`] backs
//! unit tests, [`LocalFirstStore`] is the real orchestration over two
//! [`object_store::ObjectStore`] backends (an S3 origin + a local cache). The
//! backend-construction glue ([`LocalFirstStore::from_config`]) is the only
//! untested seam; the local-first logic is exercised with in-memory backends.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use object_store::ObjectStore;
use object_store::path::Path as ObjPath;

/// A storage operation failure. `NotFound` is modelled explicitly so callers can
/// distinguish a missing object (e.g. an already-deleted score) from a backend
/// fault.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("object not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

/// Result alias for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;

/// The storage surface the rest of the backend depends on. Keys are opaque
/// object keys (e.g. `user-scores/{owner_id}/{uuid}.mxl`).
#[async_trait]
pub trait ObjectStorage: Send + Sync {
    /// Stores `bytes` at `key`, overwriting any existing object.
    async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<()>;
    /// Fetches the object at `key`, or [`StorageError::NotFound`].
    async fn get(&self, key: &str) -> Result<Vec<u8>>;
    /// Removes the object at `key`. Deleting a missing key is a no-op (idempotent).
    async fn delete(&self, key: &str) -> Result<()>;
    /// Total byte size of the object at `key` (for `Content-Length` / `Content-Range`
    /// when serving large assets), or [`StorageError::NotFound`].
    async fn size(&self, key: &str) -> Result<u64>;
    /// Fetches the half-open byte range `[range.start, range.end)` of `key`, so large
    /// assets (SoundFonts) can be delivered in parts rather than buffered whole. The
    /// range is clamped to the object; an out-of-range start yields an empty slice.
    async fn get_range(&self, key: &str, range: std::ops::Range<usize>) -> Result<Vec<u8>>;
    /// Every object key under `prefix`, across **all** backends this store reads,
    /// deduplicated — so a caller enumerating the corpus sees an object whether it
    /// lives locally, off-box, or both.
    ///
    /// Added for corpus↔catalog reconciliation (change:
    /// fix-crawler-corpus-isolation), which must find objects no catalog row
    /// references. Keys come back in no particular order.
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}

/// In-memory [`ObjectStorage`] for unit tests — no disk, no network.
#[derive(Default)]
pub struct FakeStore {
    objects: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl FakeStore {
    /// Number of stored objects (test assertions).
    pub fn len(&self) -> usize {
        self.objects.lock().expect("fake store lock").len()
    }

    /// Whether the store holds no objects.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether an object exists at `key` (test assertions).
    pub fn contains(&self, key: &str) -> bool {
        self.objects
            .lock()
            .expect("fake store lock")
            .contains_key(key)
    }
}

#[async_trait]
impl ObjectStorage for FakeStore {
    async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<()> {
        self.objects
            .lock()
            .expect("fake store lock")
            .insert(key.to_string(), bytes);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        self.objects
            .lock()
            .expect("fake store lock")
            .get(key)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(key.to_string()))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.objects.lock().expect("fake store lock").remove(key);
        Ok(())
    }

    async fn size(&self, key: &str) -> Result<u64> {
        self.objects
            .lock()
            .expect("fake store lock")
            .get(key)
            .map(|v| v.len() as u64)
            .ok_or_else(|| StorageError::NotFound(key.to_string()))
    }

    async fn get_range(&self, key: &str, range: std::ops::Range<usize>) -> Result<Vec<u8>> {
        let guard = self.objects.lock().expect("fake store lock");
        let v = guard
            .get(key)
            .ok_or_else(|| StorageError::NotFound(key.to_string()))?;
        Ok(slice_range(v, range))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        Ok(self
            .objects
            .lock()
            .expect("fake store lock")
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}

/// Clamp a half-open range to `data` and return the slice (empty if start ≥ len).
fn slice_range(data: &[u8], range: std::ops::Range<usize>) -> Vec<u8> {
    let end = range.end.min(data.len());
    let start = range.start.min(end);
    data[start..end].to_vec()
}

/// Connection parameters for the S3-compatible origin bucket.
#[derive(Debug, Clone)]
pub struct S3Params {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    /// Allow plain HTTP (a local MinIO in dev; keep false in prod).
    pub allow_http: bool,
}

/// Real [`ObjectStorage`]: an S3 origin fronted by a local-disk warm cache.
///
/// - `put` / `delete` target the **origin** (S3), the durable source of truth;
///   `delete` also best-effort clears the local cache.
/// - `get` reads the **local** cache first and, on a miss, pulls from the origin
///   and writes the object through to the local cache (so the next read is local).
pub struct LocalFirstStore {
    local: Arc<dyn ObjectStore>,
    origin: Arc<dyn ObjectStore>,
}

impl LocalFirstStore {
    /// Wraps two backends directly — a local cache and the durable origin. Used
    /// by tests with in-memory backends; production uses [`Self::from_config`].
    pub fn new(local: Arc<dyn ObjectStore>, origin: Arc<dyn ObjectStore>) -> Self {
        Self { local, origin }
    }

    /// Builds the production store: a `LocalFileSystem` rooted at `local_root`
    /// (created if absent) as the cache, and an S3 bucket as the origin.
    pub fn from_config(local_root: &str, s3: &S3Params) -> Result<Self> {
        std::fs::create_dir_all(local_root)
            .map_err(|e| StorageError::Backend(anyhow::anyhow!("local root {local_root}: {e}")))?;
        let local = object_store::local::LocalFileSystem::new_with_prefix(local_root)
            .map_err(|e| StorageError::Backend(e.into()))?;
        let origin = object_store::aws::AmazonS3Builder::new()
            .with_bucket_name(&s3.bucket)
            .with_endpoint(&s3.endpoint)
            .with_region(&s3.region)
            .with_access_key_id(&s3.access_key)
            .with_secret_access_key(&s3.secret_key)
            .with_allow_http(s3.allow_http)
            .build()
            .map_err(|e| StorageError::Backend(e.into()))?;
        Ok(Self {
            local: Arc::new(local),
            origin: Arc::new(origin),
        })
    }
}

/// Maps an `object_store` error to our [`StorageError`], preserving `NotFound`.
fn map_err(key: &str, e: object_store::Error) -> StorageError {
    match e {
        object_store::Error::NotFound { .. } => StorageError::NotFound(key.to_string()),
        other => StorageError::Backend(other.into()),
    }
}

async fn put_to(store: &Arc<dyn ObjectStore>, key: &str, bytes: Bytes) -> Result<()> {
    let path = ObjPath::from(key);
    store
        .put(&path, bytes.into())
        .await
        .map(|_| ())
        .map_err(|e| map_err(key, e))
}

async fn get_from(store: &Arc<dyn ObjectStore>, key: &str) -> Result<Bytes> {
    let path = ObjPath::from(key);
    let got = store.get(&path).await.map_err(|e| map_err(key, e))?;
    got.bytes().await.map_err(|e| map_err(key, e))
}

async fn head_size(store: &Arc<dyn ObjectStore>, key: &str) -> Result<u64> {
    let path = ObjPath::from(key);
    let meta = store.head(&path).await.map_err(|e| map_err(key, e))?;
    Ok(meta.size as u64)
}

#[async_trait]
impl ObjectStorage for LocalFirstStore {
    async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<()> {
        // Durable origin is the source of truth for user uploads (design 4).
        put_to(&self.origin, key, Bytes::from(bytes)).await
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        match get_from(&self.local, key).await {
            Ok(b) => Ok(b.to_vec()),
            Err(StorageError::NotFound(_)) => {
                // Local miss: pull from the durable origin and warm the cache so
                // the next read is local. A cache-write failure must not fail the
                // read — the bytes are already in hand.
                let bytes = get_from(&self.origin, key).await?;
                let _ = put_to(&self.local, key, bytes.clone()).await;
                Ok(bytes.to_vec())
            }
            Err(e) => Err(e),
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = ObjPath::from(key);
        // Origin is authoritative; a missing object there is not an error.
        match self.origin.delete(&path).await {
            Ok(()) => {}
            Err(object_store::Error::NotFound { .. }) => {}
            Err(e) => return Err(StorageError::Backend(e.into())),
        }
        // Best-effort cache eviction.
        let _ = self.local.delete(&path).await;
        Ok(())
    }

    async fn size(&self, key: &str) -> Result<u64> {
        match head_size(&self.local, key).await {
            Ok(n) => Ok(n),
            Err(StorageError::NotFound(_)) => head_size(&self.origin, key).await,
            Err(e) => Err(e),
        }
    }

    async fn get_range(&self, key: &str, range: std::ops::Range<usize>) -> Result<Vec<u8>> {
        // Serve the range from the local cache; on a miss, warm the whole object from
        // the origin (so subsequent ranges are local) and slice it. Warming keeps the
        // read path identical to `get`, at the cost of pulling the whole object once.
        let path = ObjPath::from(key);
        match self.local.get_range(&path, range.clone()).await {
            Ok(b) => Ok(b.to_vec()),
            Err(object_store::Error::NotFound { .. }) => {
                let whole = get_from(&self.origin, key).await?;
                let _ = put_to(&self.local, key, whole.clone()).await;
                Ok(slice_range(&whole, range))
            }
            Err(e) => Err(map_err(key, e)),
        }
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        // Union of both backends: an object may have been evicted from the cache
        // but still live in the origin, or warmed locally and (wrongly) absent
        // off-box. Reconciliation must see it either way.
        let mut keys = std::collections::BTreeSet::new();
        for backend in [&self.origin, &self.local] {
            keys.extend(list_keys(backend, prefix).await?);
        }
        Ok(keys.into_iter().collect())
    }
}

/// Collects every key under `prefix` from one `object_store` backend.
async fn list_keys(store: &Arc<dyn ObjectStore>, prefix: &str) -> Result<Vec<String>> {
    use futures_util::StreamExt;
    let path = ObjPath::from(prefix);
    let mut out = Vec::new();
    let mut stream = store.list(Some(&path));
    while let Some(meta) = stream.next().await {
        match meta {
            Ok(m) => out.push(m.location.to_string()),
            // A prefix that has never been written is empty, not an error.
            Err(object_store::Error::NotFound { .. }) => break,
            Err(e) => return Err(StorageError::Backend(e.into())),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use object_store::memory::InMemory;

    fn local_first() -> (LocalFirstStore, Arc<dyn ObjectStore>, Arc<dyn ObjectStore>) {
        let local: Arc<dyn ObjectStore> = Arc::new(InMemory::new());
        let origin: Arc<dyn ObjectStore> = Arc::new(InMemory::new());
        let store = LocalFirstStore::new(local.clone(), origin.clone());
        (store, local, origin)
    }

    async fn count(store: &Arc<dyn ObjectStore>, key: &str) -> bool {
        store.get(&ObjPath::from(key)).await.is_ok()
    }

    // --- FakeStore -------------------------------------------------------

    #[tokio::test]
    async fn fake_round_trips_and_reports_missing() {
        let s = FakeStore::default();
        assert!(s.is_empty());
        s.put("k", b"hello".to_vec()).await.unwrap();
        assert!(s.contains("k"));
        assert_eq!(s.get("k").await.unwrap(), b"hello");
        assert_eq!(s.len(), 1);
        match s.get("missing").await {
            Err(StorageError::NotFound(k)) => assert_eq!(k, "missing"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fake_delete_is_idempotent() {
        let s = FakeStore::default();
        s.put("k", b"x".to_vec()).await.unwrap();
        s.delete("k").await.unwrap();
        assert!(!s.contains("k"));
        s.delete("k").await.unwrap(); // deleting a missing key is fine
    }

    // --- LocalFirstStore -------------------------------------------------

    #[tokio::test]
    async fn put_targets_origin_not_local() {
        let (store, local, origin) = local_first();
        store
            .put("user-scores/u/1.mxl", b"bytes".to_vec())
            .await
            .unwrap();
        assert!(count(&origin, "user-scores/u/1.mxl").await, "origin has it");
        assert!(
            !count(&local, "user-scores/u/1.mxl").await,
            "local not written on put"
        );
    }

    #[tokio::test]
    async fn get_serves_local_hit_without_touching_origin() {
        let (store, local, _origin) = local_first();
        // Simulate the crawler corpus already on local disk.
        local
            .put(
                &ObjPath::from("safe/ab/x.mxl"),
                Bytes::from_static(b"corpus").into(),
            )
            .await
            .unwrap();
        assert_eq!(store.get("safe/ab/x.mxl").await.unwrap(), b"corpus");
    }

    #[tokio::test]
    async fn get_falls_back_to_origin_and_warms_local() {
        let (store, local, origin) = local_first();
        // Only on S3 (a fresh upload, or an empty/rebuilt box).
        origin
            .put(
                &ObjPath::from("user-scores/u/1.mxl"),
                Bytes::from_static(b"data").into(),
            )
            .await
            .unwrap();
        assert!(
            !count(&local, "user-scores/u/1.mxl").await,
            "not cached yet"
        );
        assert_eq!(store.get("user-scores/u/1.mxl").await.unwrap(), b"data");
        // The fallback warmed the local cache.
        assert!(
            count(&local, "user-scores/u/1.mxl").await,
            "warmed after fallback"
        );
    }

    #[tokio::test]
    async fn size_and_range_local_hit() {
        let (store, local, _origin) = local_first();
        local
            .put(
                &ObjPath::from("assets/font.sf2"),
                Bytes::from_static(b"0123456789").into(),
            )
            .await
            .unwrap();
        assert_eq!(store.size("assets/font.sf2").await.unwrap(), 10);
        assert_eq!(
            store.get_range("assets/font.sf2", 2..5).await.unwrap(),
            b"234"
        );
        // Range clamps to the object rather than erroring.
        assert_eq!(
            store.get_range("assets/font.sf2", 8..100).await.unwrap(),
            b"89"
        );
    }

    #[tokio::test]
    async fn range_falls_back_to_origin_and_warms() {
        let (store, local, origin) = local_first();
        origin
            .put(
                &ObjPath::from("assets/font.sf2"),
                Bytes::from_static(b"abcdefgh").into(),
            )
            .await
            .unwrap();
        assert!(!count(&local, "assets/font.sf2").await, "not cached yet");
        assert_eq!(
            store.get_range("assets/font.sf2", 1..4).await.unwrap(),
            b"bcd"
        );
        // The range miss warmed the whole object into the local cache.
        assert!(
            count(&local, "assets/font.sf2").await,
            "warmed after range miss"
        );
        assert_eq!(store.size("assets/font.sf2").await.unwrap(), 8);
    }

    #[tokio::test]
    async fn fake_size_and_range() {
        let s = FakeStore::default();
        s.put("k", b"hello world".to_vec()).await.unwrap();
        assert_eq!(s.size("k").await.unwrap(), 11);
        assert_eq!(s.get_range("k", 0..5).await.unwrap(), b"hello");
        assert!(matches!(
            s.size("missing").await,
            Err(StorageError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_missing_everywhere_is_not_found() {
        let (store, _local, _origin) = local_first();
        match store.get("nope").await {
            Err(StorageError::NotFound(k)) => assert_eq!(k, "nope"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn delete_clears_both_stores() {
        let (store, local, origin) = local_first();
        let key = "user-scores/u/1.mxl";
        origin
            .put(&ObjPath::from(key), Bytes::from_static(b"d").into())
            .await
            .unwrap();
        local
            .put(&ObjPath::from(key), Bytes::from_static(b"d").into())
            .await
            .unwrap();
        store.delete(key).await.unwrap();
        assert!(!count(&origin, key).await);
        assert!(!count(&local, key).await);
        store.delete(key).await.unwrap(); // idempotent
    }
}
