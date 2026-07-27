---
name: rust-testing
description: Test-double convention for the Rust backend + engine crates. Use when writing or editing any Rust test, or choosing a test double for a trait dependency (repo, port, service, client, storage). DEFAULT to mockall generated mocks (#[automock] / mock!) injected through the existing trait-object seams; hand-written fake structs are reserved for special cases (documented below).
metadata:
  author: cymbra
  version: "1.0"
---

# Rust test doubles — mockall by default (Cymbra)

Default to **mockall generated mocks** for a trait dependency's test double, **not**
hand-written fake structs. Collaborators are already behind trait seams
(`Arc<dyn Repo>`, ports), so a mock injects exactly where a fake did.

Requires the `mockall` dev-dependency.

## Generate a mock

Put `#[automock]` on the trait (compiled only under `cfg(test)` so it never ships):

```rust
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait::async_trait] // keep automock ABOVE async_trait
pub trait CatalogSearchRepo: Send + Sync {
    async fn object_key(&self, id: &str, include_unvalidated: bool) -> Result<Option<String>>;
    async fn set_moderation_status(&self, id: &str, status: &str, reviewer: &str) -> Result<bool>;
}
```

For a foreign trait you can't annotate, use `mock! { ... }`.

## Use in a test

```rust
use mockall::predicate::*;

let mut repo = MockCatalogSearchRepo::new();
repo.expect_set_moderation_status()
    .with(eq("id-1"), eq("accepted"), eq("mod-1"))
    .times(1)
    .returning(|_, _, _| Ok(true));

let module = ScoreModule::new(Arc::new(repo), /* … */);
module.set_moderation_status("mod-1", "id-1", "accepted").await.unwrap();
// `.times(1)` + `.with(...)` assert the interaction; no manual bookkeeping.
```

- Set expectations with `expect_<method>()` + `.with(...)`/`.returning(...)`/
  `.times(...)`; mockall verifies call counts on drop.
- Inject the `MockFoo` where the real `dyn Foo` goes (`Arc::new(mock)`).

## Special cases — a hand-written fake struct is allowed when…

Use a fake only when a mock would be worse, and say why in a comment:
- **Behavioural in-memory adapters**: the double must *behave* across calls
  (e.g. a repo that dedups by content hash, an object store that actually
  stores/returns bytes), so state-based tests read better than long
  `expect_*` chains. The existing `FakeCatalogRepo`/`FakeStore`/
  `FakeCatalogSearchRepo` are legitimate here.
- **Fakes the production code ships** (an in-memory adapter reused by tests).
- **Pure functions / value types**: no double needed at all — test directly.

Otherwise: mockall.

## Notes

- `mockall` is a **dev-dependency** (`mockall = { workspace = true }` under
  `[dev-dependencies]`); `#[cfg_attr(test, automock)]` keeps it out of release builds.
- Async traits use `async_trait`; put `#[automock]` **above** `#[async_trait]`.
- The thin I/O adapters excluded from the coverage gate (`pg.rs`, etc.) are still
  covered by integration tests against real infra, not mocks.

## Checklist

- [ ] Trait doubles are `mockall` mocks (`#[automock]` / `mock!`), not new hand fakes.
- [ ] Mocks injected through the existing trait-object seam.
- [ ] Expectations use `expect_*` + `.with`/`.returning`/`.times`.
- [ ] `mockall` is a dev-dependency; `#[automock]` gated on `cfg(test)`.
- [ ] Any hand-written fake carries a one-line justification (a special case above).
